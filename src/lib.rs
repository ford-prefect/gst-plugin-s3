// Copyright (C) 2017 Author: Arun Raghavan <arun@arunraghavan.net>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#![crate_type = "cdylib"]

extern crate futures;
extern crate rusoto_core;
extern crate rusoto_s3;
extern crate url;

extern crate gobject_subclass;
#[macro_use]
extern crate gstreamer as gst;
extern crate gstreamer_base as gst_base;
#[macro_use]
extern crate gst_plugin;

mod s3url;
mod s3src;

fn plugin_init(plugin: &gst::Plugin) -> bool {
    s3src::register(plugin);
    true
}

plugin_define!(
    b"s3src\0",
    b"Amazon S3 Plugin\0",
    plugin_init,
    b"1.0\0",
    b"MIT/X11\0",
    b"s3\0",
    b"s3\0",
    b"https://github.com/ford-prefect/gst-plugin-s3\0",
    b"2017-04-17\0"
);
