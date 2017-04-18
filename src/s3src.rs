// Copyright (C) 2017 Author: Arun Raghavan <arun@arunraghavan.net>
//
// This library is free software; you can redistribute it and/or
// modify it under the terms of the GNU Library General Public
// License as published by the Free Software Foundation; either
// version 3 of the License.
//
// This library is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
// Library General Public License for more details.
//
// You should have received a copy of the GNU Library General Public
// License along with this library; if not, write to the
// Free Software Foundation, Inc., 51 Franklin St, Fifth Floor,
// Boston, MA 02110-1301, USA.

use hyper;
use rusoto::default_tls_client;
use rusoto::DefaultCredentialsProvider;
use rusoto::s3::*;

use slog::Logger;
use url::Url;

use gst_plugin::buffer::*;
use gst_plugin::error::*;
use gst_plugin::log::*;
use gst_plugin::source::*;
use gst_plugin::utils::*;

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
    logger: Logger,
}

impl S3Src {
    pub fn new(element: Element) -> S3Src {
        S3Src {
            state: StreamingState::Stopped,
            logger: Logger::root(GstDebugDrain::new(Some(&element),
                                                    "s3src",
                                                    0,
                                                    "Amazon S3 Source"),
                                 o!()),
        }
    }

    pub fn new_boxed(element: Element) -> Box<Source> {
        Box::new(S3Src::new(element))
    }

    fn connect(self: &S3Src, url: &GstS3Url) -> Result<GstS3Client, ErrorMessage> {
        let dispatcher = default_tls_client()
            .or_else(|err| {
                         Err(error_msg!(SourceError::Failure,
                                        ["Failed to create TLs client: '{}'", err]))
                     })?;
        let provider = DefaultCredentialsProvider::new().unwrap();

        Ok(S3Client::new(dispatcher, provider, url.region))
    }

    fn head(self: &S3Src, client: &GstS3Client, url: &GstS3Url) -> Result<u64, ErrorMessage> {
        let request = HeadObjectRequest {
            bucket: url.bucket.clone(),
            key: url.object.clone(),
            version_id: url.version.clone(),
            ..Default::default()
        };

        let output = client
            .head_object(&request)
            .or_else(|err| {
                         Err(error_msg!(SourceError::OpenFailed,
                                        ["Failed to HEAD object: {}", err]))
                     })?;

        if let Some(size) = output.content_length {
            info!(self.logger, "HEAD success, content length = {}", size);
            Ok(size as u64)
        } else {
            Err(error_msg!(SourceError::OpenFailed, ["Failed to get content length"]))
        }

    }
}

impl Source for S3Src {
    fn uri_validator(&self) -> Box<UriValidator> {
        Box::new(|url: &Url| -> Result<(), UriError> {
                     parse_s3_url(url)?;
                     Ok(())
                 })
    }

    fn is_seekable(&self) -> bool {
        // FIXME
        false
    }

    fn get_size(&self) -> Option<u64> {
        match self.state {
            StreamingState::Stopped => None,
            StreamingState::Started { size: size, .. } => Some(size),
        }
    }

    fn start(&mut self, url: Url) -> Result<(), ErrorMessage> {
        let s3url = parse_s3_url(&url)
            .or_else(|err| Err(error_msg!(SourceError::NotFound, [err.to_string()])))?;

        let s3client = self.connect(&s3url)?;

        let size = self.head(&s3client, &s3url)?;

        self.state = StreamingState::Started {
            url: s3url,
            client: s3client,
            size: size,
        };

        Ok(())
    }

    fn stop(&mut self) -> Result<(), ErrorMessage> {
        self.state = StreamingState::Stopped;

        Ok(())
    }

    fn fill(&mut self, offset: u64, length: u32, buffer: &mut Buffer) -> Result<(), FlowError> {
        // FIXME
        Ok(())
    }

    fn seek(&mut self, start: u64, stop: Option<u64>) -> Result<(), ErrorMessage> {
        // FIXME
        Ok(())
    }
}
