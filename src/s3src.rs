// Copyright (C) 2017 Author: Arun Raghavan <arun@arunraghavan.net>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use hyper;
use rusoto_core::{DefaultCredentialsProvider, default_tls_client};
use rusoto_s3::*;

use url::Url;

use gst;
use gst::buffer::*;
use gst_plugin::error::*;

use gst_plugin_simple::UriValidator;
use gst_plugin_simple::source::*;

use s3url::*;

type GstS3Client = S3Client<DefaultCredentialsProvider, hyper::client::Client>;

enum StreamingState {
    Stopped,
    Started {
        url: GstS3Url,
        client: GstS3Client,
        size: u64,
    },
}

pub struct S3Src {
    state: StreamingState,
    cat: gst::DebugCategory,
}

impl S3Src {
    pub fn new(_: &RsBaseSrc) -> S3Src {
        S3Src {
            state: StreamingState::Stopped,
            cat: gst::DebugCategory::new("s3src",
                                         gst::DebugColorFlags::empty(),
                                         "Amazon S3 Source"),
        }
    }

    pub fn new_boxed(src: &RsBaseSrc) -> Box<SourceImpl> {
        Box::new(S3Src::new(src))
    }

    fn connect(self: &S3Src, url: &GstS3Url) -> Result<GstS3Client, ErrorMessage> {
        let dispatcher = default_tls_client()
            .or_else(|err| {
                         Err(error_msg!(gst::LibraryError::Failed,
                                        ["Failed to create TLs client: '{}'", err]))
                     })?;
        let provider = DefaultCredentialsProvider::new().unwrap();

        Ok(S3Client::new(dispatcher, provider, url.region.clone()))
    }

    fn head(self: &S3Src,
            src: &RsBaseSrc,
            client: &GstS3Client,
            url: &GstS3Url)
            -> Result<u64, ErrorMessage> {
        let request = HeadObjectRequest {
            bucket: url.bucket.clone(),
            key: url.object.clone(),
            version_id: url.version.clone(),
            ..Default::default()
        };

        let output = client
            .head_object(&request)
            .or_else(|err| {
                         Err(error_msg!(gst::ResourceError::NotFound,
                                        ["Failed to HEAD object: {}", err]))
                     })?;

        if let Some(size) = output.content_length {
            gst_info!(self.cat,
                      obj: src,
                      "HEAD success, content length = {}",
                      size);
            Ok(size as u64)
        } else {
            Err(error_msg!(gst::ResourceError::Read, ["Failed to get content length"]))
        }

    }

    fn get(self: &S3Src,
           src: &RsBaseSrc,
           offset: u64,
           length: u64)
           -> Result<Vec<u8>, ErrorMessage> {
        let (url, client) = match self.state {
            StreamingState::Started {
                ref url,
                ref client,
                ..
            } => (url, client),
            StreamingState::Stopped => {
                return Err(error_msg!(gst::LibraryError::Failed, ["Cannot GET before start()"]));
            }
        };

        let request = GetObjectRequest {
            bucket: url.bucket.clone(),
            key: url.object.clone(),
            range: Some(format!("bytes={}-{}", offset, offset + length - 1)),
            version_id: url.version.clone(),
            ..Default::default()
        };

        gst_debug!(self.cat,
                   obj: src,
                   "Requesting range: {}-{}",
                   offset,
                   offset + length - 1);

        let output =
            client
                .get_object(&request)
                .or_else(|err| Err(error_msg!(gst::ResourceError::Read, [err.to_string()])))?;

        gst_debug!(self.cat,
                   obj: src,
                   "Read {} bytes",
                   output.content_length.unwrap());

        let mut body: Vec<u8> = Vec::new();

        output
            .body
            .unwrap()
            .read_to_end(&mut body)
            .or_else(|err| Err(error_msg!(gst::ResourceError::Read, [err.to_string()])))?;

        Ok(body)
    }
}

impl SourceImpl for S3Src {
    fn uri_validator(&self) -> Box<UriValidator> {
        Box::new(|url: &Url| -> Result<(), UriError> {
                     parse_s3_url(url)?;
                     Ok(())
                 })
    }

    fn is_seekable(&self, _: &RsBaseSrc) -> bool {
        true
    }

    fn get_size(&self, _: &RsBaseSrc) -> Option<u64> {
        match self.state {
            StreamingState::Stopped => None,
            StreamingState::Started { size, .. } => Some(size),
        }
    }

    fn start(&mut self, src: &RsBaseSrc, url: Url) -> Result<(), ErrorMessage> {
        if let StreamingState::Started { .. } = self.state {
            return Err(error_msg!(gst::LibraryError::Failed,
                                  ["Cannot start() while already started"]));
        }

        let s3url =
            parse_s3_url(&url)
                .or_else(|err| Err(error_msg!(gst::ResourceError::Failed, [err.to_string()])))?;

        let s3client = self.connect(&s3url)?;

        let size = self.head(src, &s3client, &s3url)?;

        self.state = StreamingState::Started {
            url: s3url,
            client: s3client,
            size: size,
        };

        Ok(())
    }

    fn stop(&mut self, _: &RsBaseSrc) -> Result<(), ErrorMessage> {
        if let StreamingState::Stopped = self.state {
            return Err(error_msg!(gst::LibraryError::Failed, ["Cannot stop() before start()"]));
        }

        self.state = StreamingState::Stopped;

        Ok(())
    }

    fn fill(&mut self,
            src: &RsBaseSrc,
            offset: u64,
            length: u32,
            buffer: &mut BufferRef)
            -> Result<(), FlowError> {
        // FIXME: sanity check on offset and length
        let data = self.get(src, offset, length as u64)
            .or_else(|err| Err(FlowError::Error(err)))?;

        buffer
            .copy_from_slice(0, data.as_slice())
            .or_else(|copied| {
                         Err(FlowError::Error(error_msg!(gst::ResourceError::Read,
                                                         ["Read {} bytes, but buffer has {} bytes",
                                                          data.len(),
                                                          copied])))
                     })?;
        buffer.set_size(data.len());

        Ok(())
    }

    fn seek(&mut self, _: &RsBaseSrc, _: u64, _: Option<u64>) -> Result<(), ErrorMessage> {
        Ok(())
    }
}
