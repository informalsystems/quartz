#![allow(clippy::unwrap_used, unused_qualifications)]

pub mod quartz {
    include!(concat!("prost/", "quartz.rs"));
}
