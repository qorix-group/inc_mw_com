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

use com_api::*;
use com_api_runtime_iceoryx::{
    IceoryxAdapter, IceoryxConsumerBuilder, IceoryxProducerBuilder, IceoryxPublisher,
    IceoryxSubscribable,
};
use iceoryx2::prelude::*;

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
impl InterfaceConcept for VehicleInterface {}
unsafe impl ZeroCopySend for VehicleInterface {}

pub struct AnotherInterface {}

impl InterfaceConcept for AnotherInterface {}

pub struct VehicleProducer {
    pub left_tire: IceoryxPublisher<Tire>,
    pub exhaust: IceoryxPublisher<Exhaust>,
}

impl ProducerConcept for VehicleProducer {
    type Interface = VehicleInterface;
    type OfferedProducer = VehicleOfferedProducer;

    fn offer(self) -> com_api::Result<Self::OfferedProducer> {
        Ok(VehicleOfferedProducer {
            left_tire: self.left_tire,
            exhaust: self.exhaust,
        })
    }
}

pub struct VehicleOfferedProducer {
    pub left_tire: IceoryxPublisher<Tire>,
    pub exhaust: IceoryxPublisher<Exhaust>,
}

impl OfferedProducerConcept for VehicleOfferedProducer {
    type Interface = VehicleInterface;
    type Producer = VehicleProducer;

    fn unoffer(self) -> Self::Producer {
        VehicleProducer {
            left_tire: self.left_tire,
            exhaust: self.exhaust,
        }
    }
}

impl BuilderConcept<VehicleProducer> for IceoryxProducerBuilder<VehicleInterface> {
    fn build(self) -> com_api::Result<VehicleProducer> {
        let left_tire_name = format!("{}/left_tire", self.instance_specifier.specifier);
        let left_tire = IceoryxPublisher::new(
            self.node
                .service_builder(&ServiceName::new(left_tire_name.as_str()).unwrap())
                .publish_subscribe::<Tire>()
                .open_or_create()
                .unwrap(),
        );
        let exhaust_service_name = format!("{}/exhaust", self.instance_specifier.specifier);
        let exhaust = IceoryxPublisher::new(
            self.node
                .service_builder(&ServiceName::new(exhaust_service_name.as_str()).unwrap())
                .publish_subscribe::<Exhaust>()
                .open_or_create()
                .unwrap(),
        );
        Ok(VehicleProducer { left_tire, exhaust })
    }
}

impl ProducerBuilderConcept<VehicleInterface, IceoryxAdapter, VehicleProducer>
    for IceoryxProducerBuilder<VehicleInterface>
{
}

pub struct VehicleConsumer {
    pub left_tire: IceoryxSubscribable<Tire>,
    pub exhaust: IceoryxSubscribable<Exhaust>,
}

impl ConsumerConcept for VehicleConsumer {}

impl ConsumerBuilderConcept<VehicleInterface, IceoryxAdapter>
    for IceoryxConsumerBuilder<VehicleInterface>
{
}

impl BuilderConcept<VehicleConsumer> for IceoryxConsumerBuilder<VehicleInterface> {
    fn build(self) -> com_api::Result<VehicleConsumer> {
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
        Ok(VehicleConsumer {
            left_tire: IceoryxSubscribable::new(left_tire),
            exhaust: IceoryxSubscribable::new(exhaust),
        })
    }
}
