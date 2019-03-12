// Copyright (C) 2017 Author: Arun Raghavan <arun@arunraghavan.net>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::sync::Mutex;

use futures::{Future,Stream};
use rusoto_s3::*;

use glib::prelude::*;
use glib::subclass;
use glib::subclass::prelude::*;

use gst;
use gst::subclass::prelude::*;

use gst_base;
use gst_base::prelude::*;
use gst_base::subclass::prelude::*;

use s3url::*;

enum StreamingState {
    Stopped,
    Started {
        url: GstS3Url,
        client: S3Client,
        size: u64,
    },
}

pub struct S3Src {
    url: Mutex<Option<GstS3Url>>,
    state: Mutex<StreamingState>,
    cat: gst::DebugCategory,
}

static PROPERTIES: [subclass::Property; 1] = [subclass::Property("uri", |name| {
    glib::ParamSpec::string(
        name,
        "URI",
        "The S3 object URI",
        None,
        glib::ParamFlags::READWRITE /* + GST_PARAM_MUTABLE_READY) */
    )
})];

impl S3Src {
    fn connect(self: &S3Src, url: &GstS3Url) -> Result<S3Client, gst::ErrorMessage> {
        Ok(S3Client::new(url.region.clone()))
    }

    fn head(
        self: &S3Src,
        src: &gst_base::BaseSrc,
        client: &S3Client,
        url: &GstS3Url,
    ) -> Result<u64, gst::ErrorMessage> {
        let request = HeadObjectRequest {
            bucket: url.bucket.clone(),
            key: url.object.clone(),
            version_id: url.version.clone(),
            ..Default::default()
        };

        let output = client.head_object(request).sync().or_else(|err| {
            Err(gst_error_msg!(
                gst::ResourceError::NotFound,
                ["Failed to HEAD object: {}", err]
            ))
        })?;

        if let Some(size) = output.content_length {
            gst_info!(
                self.cat,
                obj: src,
                "HEAD success, content length = {}",
                size
            );
            Ok(size as u64)
        } else {
            Err(gst_error_msg!(
                gst::ResourceError::Read,
                ["Failed to get content length"]
            ))
        }
    }

    fn get(
        self: &S3Src,
        src: &gst_base::BaseSrc,
        offset: u64,
        length: u64,
    ) -> Result<Vec<u8>, gst::ErrorMessage> {
        let state = self.state.lock().unwrap();

        let (url, client) = match *state {
            StreamingState::Started {
                ref url,
                ref client,
                ..
            } => (url, client),
            StreamingState::Stopped => {
                return Err(gst_error_msg!(
                    gst::LibraryError::Failed,
                    ["Cannot GET before start()"]
                ));
            }
        };

        let request = GetObjectRequest {
            bucket: url.bucket.clone(),
            key: url.object.clone(),
            range: Some(format!("bytes={}-{}", offset, offset + length - 1)),
            version_id: url.version.clone(),
            ..Default::default()
        };

        gst_debug!(
            self.cat,
            obj: src,
            "Requesting range: {}-{}",
            offset,
            offset + length - 1
        );

        let output = client.get_object(request).sync().or_else(|err| {
            Err(gst_error_msg!(gst::ResourceError::Read, ["Could not read: {}", err]))
        })?;

        gst_debug!(
            self.cat,
            obj: src,
            "Read {} bytes",
            output.content_length.unwrap()
        );

        let body = output.body.unwrap().concat2().wait().or_else(|err| {
            Err(gst_error_msg!(gst::ResourceError::Read, ["Could not read: {}", err]))
        })?;

        Ok(body)
    }
}

impl ObjectSubclass for S3Src {
    const NAME: &'static str = "S3Src";
    type ParentType = gst_base::BaseSrc;
    type Instance = gst::subclass::ElementInstanceStruct<Self>;
    type Class = subclass::simple::ClassStruct<Self>;

    glib_object_subclass!();

    fn new() -> Self {
        Self {
            url: Mutex::new(None),
            state: Mutex::new(StreamingState::Stopped),
            cat: gst::DebugCategory::new(
                "s3src",
                gst::DebugColorFlags::empty(),
                "Amazon S3 Source",
            ),
        }
    }

    fn class_init(klass: &mut subclass::simple::ClassStruct<Self>) {
        klass.set_metadata(
            "Amazon S3 source",
            "Source/Network",
            "Reads an object from Amazon S3",
            "Arun Raghavan <arun@arunraghavan.net>",
        );

        let caps = gst::Caps::new_any();
        let src_pad_template = gst::PadTemplate::new(
            "src",
            gst::PadDirection::Src,
            gst::PadPresence::Always,
            &caps,
        ).unwrap();
        klass.add_pad_template(src_pad_template);

        klass.install_properties(&PROPERTIES);
    }
}

impl ObjectImpl for S3Src {
    glib_object_impl!();

    fn set_property(&self, obj: &glib::Object, id: usize, value: &glib::Value) {
        let prop = &PROPERTIES[id as usize];
        let basesrc = obj.downcast_ref::<gst_base::BaseSrc>().unwrap();

        match *prop {
            subclass::Property("uri", ..) => {
                let url_str = value.get().unwrap();
                let mut url = self.url.lock().unwrap();

                *url = match parse_s3_url(url_str) {
                    Ok(url) => Some(url),
                    Err(err) => {
                        gst_error!(self.cat, obj: basesrc, "Could not parser uri {}: {}", url_str, err);
                        None
                    }
                }
            },
            _ => unimplemented!()
        }
    }

    fn get_property(&self, _: &glib::Object, id: usize) -> Result<glib::Value, ()> {
        let prop = &PROPERTIES[id as usize];

        match *prop {
            subclass::Property("uri", ..) => {
                let url = match *self.url.lock().unwrap() {
                    Some(ref url) => url.to_string(),
                    None => "".to_string()
                };

                Ok(url.to_value())
            },
            _ => unimplemented!()
        }
    }

    fn constructed(&self, obj: &glib::Object) {
        self.parent_constructed(obj);

        let basesrc = obj.downcast_ref::<gst_base::BaseSrc>().unwrap();
        basesrc.set_format(gst::Format::Bytes);
        /* Set a larger default blocksize to make read more efficient */
        basesrc.set_blocksize(262144);
    }
}

impl ElementImpl for S3Src {
    // No overrides
}

impl BaseSrcImpl for S3Src {
    fn is_seekable(&self, _: &gst_base::BaseSrc) -> bool {
        true
    }

    fn get_size(&self, _: &gst_base::BaseSrc) -> Option<u64> {
        match *self.state.lock().unwrap() {
            StreamingState::Stopped => None,
            StreamingState::Started { size, .. } => Some(size),
        }
    }

    fn start(&self, src: &gst_base::BaseSrc) -> Result<(), gst::ErrorMessage> {
        let mut state = self.state.lock().unwrap();

        if let StreamingState::Started { .. } = *state {
            unreachable!("S3Src is already started");
        }

        let s3url = match *self.url.lock().unwrap() {
            Some(ref url) => url.clone(),
            None => {
                return Err(gst_error_msg!(
                    gst::ResourceError::Settings,
                    ["Cannot start without a URL being set"]
                ));
            }
        };

        let s3client = self.connect(&s3url)?;

        let size = self.head(src, &s3client, &s3url)?;

        *state = StreamingState::Started {
            url: s3url,
            client: s3client,
            size: size,
        };

        Ok(())
    }

    fn stop(&self, _: &gst_base::BaseSrc) -> Result<(), gst::ErrorMessage> {
        let mut state = self.state.lock().unwrap();

        if let StreamingState::Stopped = *state {
            unreachable!("Cannot stop before start");
        }

        *state = StreamingState::Stopped;

        Ok(())
    }

   fn query(&self, src: &gst_base::BaseSrc, query: &mut gst::QueryRef) -> bool {
        match query.view_mut() {
            gst::QueryView::Scheduling(ref mut q) => {
                q.set(gst::SchedulingFlags::SEQUENTIAL | gst::SchedulingFlags::BANDWIDTH_LIMITED, 1, -1, 0);
                q.add_scheduling_modes(&[gst::PadMode::Push, gst::PadMode::Pull]);
                return true;
            }
            _ => (),
        }

        BaseSrcImplExt::parent_query(self, src, query)
    }

    fn create(
        &self,
        src: &gst_base::BaseSrc,
        offset: u64,
        length: u32,
    ) -> Result<gst::Buffer, gst::FlowError> {
        // FIXME: sanity check on offset and length
        let data = self.get(src, offset, u64::from(length));

        if data.is_err() {
            return Err(gst::FlowError::Error);
        }

        let buffer = gst::Buffer::from_mut_slice(data.unwrap());

        Ok(buffer)
    }

    /* FIXME: implement */
    fn do_seek(&self, _: &gst_base::BaseSrc, _: &mut gst::Segment) -> bool {
        true
    }
}

pub fn register(plugin: &gst::Plugin) -> Result<(), glib::BoolError> {
    gst::Element::register(plugin, "s3src", 0, S3Src::get_type())
}
