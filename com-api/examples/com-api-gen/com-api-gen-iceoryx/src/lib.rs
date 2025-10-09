// Copyright (c) 2025 Contributors to the Eclipse Foundation
//
// See the NOTICE file(s) distributed with this work for additional
// information regarding copyright ownership.
//
// This program and the accompanying materials are made available under the
// terms of the Apache License Version 2.0 which is available at
// <https://www.apache.org/licenses/LICENSE-2.0>
//
// SPDX-License-Identifier: Apache-2.0

//! This is the "generated" code for an interface that looks like this (pseudo-IDL):
//!
//! ```poor-persons-idl
//! interface Vehicle {
//!     left_tire: Event<Tire>,
//!     exhaust: Event<Exhaust>,
//!     set_indicator_state: FnMut(indicator_status: IndicatorStatus) -> Result<bool>,
//! }
//!
//! interface Another {}
//!
//! ```

#[path = "generated/vehicle_interface.rs"]
mod vehicle_interface;

use com_api::prelude::*;
use com_api_runtime_iceoryx::{
    Publisher, RuntimeImpl, SampleConsumerBuilder, SampleProducerBuilder, SubscribableImpl,
};
use iceoryx2::prelude::*;

use vehicle_interface::*;

#[derive(Debug)]
#[repr(C)]
pub struct Tire {}
unsafe impl Reloc for Tire {}
unsafe impl ZeroCopySend for Tire {}

#[derive(Debug)]
#[repr(C)]
pub struct Exhaust {}
unsafe impl Reloc for Exhaust {}
unsafe impl ZeroCopySend for Exhaust {}

#[derive(Debug)]
#[repr(C)]
pub struct VehicleInterface {}

/// Generic
impl Interface for VehicleInterface {}
unsafe impl ZeroCopySend for VehicleInterface {}

pub struct AnotherInterface {}

impl Interface for AnotherInterface {}

pub struct VehicleProducer {
    pub left_tire: Publisher<Tire>,
    pub exhaust: Publisher<Exhaust>,
    pub windows_position: Publisher<WindowsPosition>,
    pub rain_sensor: Publisher<RainSensor>,
    pub close_windows: Publisher<CloseWindows>,
}

impl Producer for VehicleProducer {
    type Interface = VehicleInterface;
    type OfferedProducer = VehicleOfferedProducer;

    fn offer(self) -> com_api::prelude::Result<Self::OfferedProducer> {
        Ok(VehicleOfferedProducer {
            left_tire: self.left_tire,
            exhaust: self.exhaust,
            windows_position: self.windows_position,
            rain_sensor: self.rain_sensor,
            close_windows: self.close_windows,
        })
    }
}

pub struct VehicleOfferedProducer {
    pub left_tire: Publisher<Tire>,
    pub exhaust: Publisher<Exhaust>,
    pub windows_position: Publisher<WindowsPosition>,
    pub rain_sensor: Publisher<RainSensor>,
    pub close_windows: Publisher<CloseWindows>,
}

impl OfferedProducer for VehicleOfferedProducer {
    type Interface = VehicleInterface;
    type Producer = VehicleProducer;

    fn unoffer(self) -> Self::Producer {
        VehicleProducer {
            left_tire: self.left_tire,
            exhaust: self.exhaust,
            windows_position: self.windows_position,
            rain_sensor: self.rain_sensor,
            close_windows: self.close_windows,
        }
    }
}

impl Builder<VehicleProducer> for SampleProducerBuilder<VehicleInterface> {
    fn build(self) -> com_api::prelude::Result<VehicleProducer> {
        let left_tire_name = format!("{}/left_tire", self.instance_specifier.specifier);
        let left_tire = Publisher::new(
            self.node
                .service_builder(&ServiceName::new(left_tire_name.as_str()).unwrap())
                .publish_subscribe::<Tire>()
                .open_or_create()
                .unwrap(),
        );
        let exhaust_service_name = format!("{}/exhaust", self.instance_specifier.specifier);
        let exhaust = Publisher::new(
            self.node
                .service_builder(&ServiceName::new(exhaust_service_name.as_str()).unwrap())
                .publish_subscribe::<Exhaust>()
                .open_or_create()
                .unwrap(),
        );
        let windows_position_service_name = format!("{}/windows_position", self.instance_specifier.specifier);
        let windows_position = Publisher::new(
            self.node
                .service_builder(&ServiceName::new(windows_position_service_name.as_str()).unwrap())
                .publish_subscribe::<WindowsPosition>()
                .open_or_create()
                .unwrap(),
        );
        let rain_sensor_service_name = format!("{}/rain_sensor", self.instance_specifier.specifier);
        let rain_sensor = Publisher::new(
            self.node
                .service_builder(&ServiceName::new(rain_sensor_service_name.as_str()).unwrap())
                .publish_subscribe::<RainSensor>()
                .open_or_create()
                .unwrap(),
        );
        let close_windows_service_name = format!("{}/close_windows", self.instance_specifier.specifier);
        let close_windows = Publisher::new(
            self.node
                .service_builder(&ServiceName::new(close_windows_service_name.as_str()).unwrap())
                .publish_subscribe::<CloseWindows>()
                .open_or_create()
                .unwrap(),
        );
        Ok(VehicleProducer { left_tire, exhaust, windows_position, rain_sensor, close_windows })
    }
}

impl ProducerBuilder<VehicleInterface, RuntimeImpl, VehicleProducer>
    for SampleProducerBuilder<VehicleInterface>
{
}

pub struct VehicleConsumer {
    pub left_tire: SubscribableImpl<Tire>,
    pub exhaust: SubscribableImpl<Exhaust>,
    pub windows_position: SubscribableImpl<WindowsPosition>,
    pub rain_sensor: SubscribableImpl<RainSensor>,
    pub close_windows: SubscribableImpl<CloseWindows>,
}

impl Consumer for VehicleConsumer {
    type Interface = VehicleInterface;
}

impl Builder<VehicleConsumer> for SampleConsumerBuilder<VehicleInterface> {
    fn build(self) -> com_api::prelude::Result<VehicleConsumer> {
        let left_tire_name = format!("{}/left_tire", self.instance_specifier.specifier);
        let left_tire = self
            .node
            .service_builder(&ServiceName::new(left_tire_name.as_str()).unwrap())
            .publish_subscribe::<Tire>()
            .open_or_create()
            .unwrap();
        let exhaust_service_name = format!("{}/exhaust", self.instance_specifier.specifier);
        let exhaust = self
            .node
            .service_builder(&ServiceName::new(exhaust_service_name.as_str()).unwrap())
            .publish_subscribe::<Exhaust>()
            .open_or_create()
            .unwrap();
        let windows_position_service_name = format!("{}/windows_position", self.instance_specifier.specifier);
        let windows_position = self
            .node
            .service_builder(&ServiceName::new(windows_position_service_name.as_str()).unwrap())
            .publish_subscribe::<WindowsPosition>()
            .open_or_create()
            .unwrap();
        let rain_sensor_service_name = format!("{}/rain_sensor", self.instance_specifier.specifier);
        let rain_sensor = self
            .node
            .service_builder(&ServiceName::new(rain_sensor_service_name.as_str()).unwrap())
            .publish_subscribe::<RainSensor>()
            .open_or_create()
            .unwrap();
        let close_windows_service_name = format!("{}/close_windows", self.instance_specifier.specifier);
        let close_windows = self
            .node
            .service_builder(&ServiceName::new(close_windows_service_name.as_str()).unwrap())
            .publish_subscribe::<CloseWindows>()
            .open_or_create()
            .unwrap();
        Ok(VehicleConsumer {
            left_tire: SubscribableImpl::new(left_tire),
            exhaust: SubscribableImpl::new(exhaust),
            windows_position: SubscribableImpl::new(windows_position),
            rain_sensor: SubscribableImpl::new(rain_sensor),
            close_windows: SubscribableImpl::new(close_windows),
        })
    }
}
