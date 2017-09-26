// Copyright (C) 2017 Author: Arun Raghavan <arun@arunraghavan.net>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::str::FromStr;

use rusoto_core::Region;
use rusoto_s3::{BucketName, ObjectKey, ObjectVersionId};
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

    let mut path =
        url.path_segments()
            .ok_or_else(|| {
                            UriError::new(UriErrorKind::BadUri,
                                          Some(format!("Invalid uri '{}'", url)))
                        })?;

    let bucket = path.next().unwrap().to_string();

    let o = path.next()
        .ok_or_else(|| {
                        UriError::new(UriErrorKind::BadUri,
                                      Some(format!("Invalid empty object/bucket '{}'", url)))
                    })?;

    let mut object = o.to_string();
    if o.is_empty() {
        return Err(UriError::new(UriErrorKind::BadUri,
                                 Some(format!("Invalid empty object/bucket '{}'", url))));
    }

    object = path.fold(object, |o, p| format!("{}/{}", o, p));

    let mut q = url.query_pairs();
    let v = q.next();
    let version;

    match v {
        Some((ref k, ref v)) if k == "version" => version = Some((*v).to_string()),
        None => version = None,
        Some(_) => {
            return Err(UriError::new(UriErrorKind::BadUri,
                                     Some(format!("Bad query, only 'version' is supported"))));
        }
    }

    if q.next() != None {
        return Err(UriError::new(UriErrorKind::BadUri,
                                 Some(format!("Extra query terms, only 'version' is supported"))));
    }

    Ok(GstS3Url {
           region: region,
           bucket: bucket,
           object: object,
           version: version,
       })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cannot_be_base() {
        let url = Url::parse("data:something").unwrap();
        assert!(parse_s3_url(&url).is_err());
    }

    #[test]
    fn invalid_scheme() {
        let url = Url::parse("file:///dev/zero").unwrap();
        assert!(parse_s3_url(&url).is_err());
    }

    #[test]
    fn bad_region() {
        let url = Url::parse("s3://atlantis-1/i-hope-we/dont-find-this").unwrap();
        assert!(parse_s3_url(&url).is_err());
    }

    #[test]
    fn no_bucket() {
        let url1 = Url::parse("s3://ap-south-1").unwrap();
        assert!(parse_s3_url(&url1).is_err());
        let url2 = Url::parse("s3://ap-south-1/").unwrap();
        assert!(parse_s3_url(&url2).is_err());
    }

    #[test]
    fn no_object() {
        let url1 = Url::parse("s3://ap-south-1/my-bucket").unwrap();
        assert!(parse_s3_url(&url1).is_err());
        let url2 = Url::parse("s3://ap-south-1/my-bucket/").unwrap();
        assert!(parse_s3_url(&url2).is_err());
    }

    #[test]
    fn valid_simple() {
        let url = Url::parse("s3://ap-south-1/my-bucket/my-object").unwrap();
        assert!(parse_s3_url(&url).is_ok());
    }

    #[test]
    fn extraneous_query() {
        let url = Url::parse("s3://ap-south-1/my-bucket/my-object?foo=bar").unwrap();
        assert!(parse_s3_url(&url).is_err());
    }

    #[test]
    fn valid_version() {
        let url = Url::parse("s3://ap-south-1/my-bucket/my-object?version=one").unwrap();
        assert!(parse_s3_url(&url).is_ok());
    }

    #[test]
    fn trailing_slash() {
        // Slashes are valid at the end of the object key
        let url = Url::parse("s3://ap-south-1/my-bucket/my-object/").unwrap();
        assert!(parse_s3_url(&url).is_ok());
    }

}
