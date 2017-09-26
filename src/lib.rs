// Copyright (C) 2017 Author: Arun Raghavan <arun@arunraghavan.net>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#![crate_type="cdylib"]

extern crate hyper;
extern crate rusoto_core;
extern crate rusoto_s3;
#[macro_use]
extern crate slog;
extern crate url;

#[macro_use]
extern crate gst_plugin;

use gst_plugin::plugin::*;
use gst_plugin::source::*;

mod s3url;
mod s3src;

use s3src::S3Src;

fn plugin_init(plugin: &Plugin) -> bool {
    source_register(plugin,
                    SourceInfo {
                        name: "s3src".into(),
                        long_name: "Amazon S3 Source".into(),
                        description: "Reads an object from an S3 region and bucket".into(),
                        classification: "Source/Network".into(),
                        author: "Arun Raghavan <arun@arunraghavan.net>".into(),
                        rank: 256,
                        create_instance: S3Src::new_boxed,
                        protocols: vec!["s3".into()],
                        push_only: true,
                    });

    true
}

plugin_define!(b"s3src\0",
               b"Amazon S3 Plugin\0",
               plugin_init,
               b"1.0\0",
               b"MIT/X11\0",
               b"s3\0",
               b"s3\0",
               b"https://github.com/ford-prefect/gst-plugin-s3\0",
               b"2017-04-17\0");
