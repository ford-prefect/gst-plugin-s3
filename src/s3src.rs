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

use slog::Logger;
use url::Url;

use gst_plugin::buffer::*;
use gst_plugin::error::*;
use gst_plugin::log::*;
use gst_plugin::source::*;
use gst_plugin::utils::*;

pub struct S3Src {
    logger: Logger,
}

impl S3Src {
    pub fn new(element: Element) -> S3Src {
        S3Src {
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
}

impl Source for S3Src {
    fn uri_validator(&self) -> Box<UriValidator> {
        Box::new(|url: &Url| -> Result<(), UriError> {
            // FIXME
            Ok(())
        })
    }

    fn is_seekable(&self) -> bool {
        // FIXME
        false
    }

    fn get_size(&self) -> Option<u64> {
        // FIXME
        None
    }

    fn start(&mut self, uri: Url) -> Result<(), ErrorMessage> {
        // FIXME
        Ok(())
    }

    fn stop(&mut self) -> Result<(), ErrorMessage> {
        // FIXME
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
