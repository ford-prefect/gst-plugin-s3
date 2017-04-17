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

use std::str::FromStr;

use rusoto::Region;
use rusoto::s3::{BucketName, ObjectKey, ObjectVersionId};
use url::Url;

use gst_plugin::error::*;

pub struct GstS3Url {
    pub region: Region,
    pub bucket: BucketName,
    pub object: ObjectKey,
    pub version: Option<ObjectVersionId>,
}

pub fn parse_s3_url(url: &Url) -> Result<GstS3Url, UriError> {
    if url.scheme() != "s3" {
        return Err(UriError::new(UriErrorKind::UnsupportedProtocol,
                                 Some(format!("Unsupported URI '{}'", url.scheme()))));
    }

    if !url.has_host() {
        return Err(UriError::new(UriErrorKind::BadUri,
                                 Some(format!("Invalid host in uri '{}'", url))));
    }

    let h = url.host_str().unwrap();
    let region = Region::from_str(h)
        .or_else(|_| {
                     Err(UriError::new(UriErrorKind::BadUri,
                                       Some(format!("Invalid region '{}'", h))))
                 })?;

    let path: Vec<&str> = url.path_segments().unwrap().collect();
    if path.len() < 2 || path.len() > 3 {
        return Err(UriError::new(UriErrorKind::BadUri, Some(format!("Invalid uri '{}'", url))));
    }

    let bucket = path[0].to_string();
    let object = path[1].to_string();

    if path[0].is_empty() || path[1].is_empty() {
        return Err(UriError::new(UriErrorKind::BadUri,
                                 Some(format!("Invalid empty object/bucket '{}'", url))));
    }

    // Gets the string inside the Some (which can be), or an empty string on None
    let ref v = path.get(2).map(|s| s.to_string()).unwrap_or_default();
    let version;

    if v.is_empty() {
        version = None;
    } else {
        version = Some(v.clone());
    }

    Ok(GstS3Url {
           region: region,
           bucket: bucket,
           object: object,
           version: version,
       })
}
