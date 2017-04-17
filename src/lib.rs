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

#![crate_type="cdylib"]

extern crate rusoto;
#[macro_use]
extern crate slog;
extern crate url;

#[macro_use]
extern crate gst_plugin;

use gst_plugin::plugin::*;
use gst_plugin::source::*;

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
               b"LGPL\0",
               b"s3\0",
               b"s3\0",
               b"https://github.com/ford-prefect/gst-plugin-s3\0",
               b"2017-04-17\0");
