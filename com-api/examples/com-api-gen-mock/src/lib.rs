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
#![allow(unused_imports)]
use com_api::*;
use com_api_runtime_mock::{MockRuntimeImpl, Sample, SampleConsumerBuilder, SampleProducerBuilder};

#[derive(Debug)]
pub struct Tire {}
unsafe impl Reloc for Tire {}

pub struct Exhaust {}
unsafe impl Reloc for Exhaust {}

#[derive(Debug)]
pub struct VehicleInterface {}

/// Generic
impl Interface for VehicleInterface {
    type ProducerType = VehicleProducer;
    type ConsumerType = VehicleConsumer;

    type RuntimeType = MockRuntimeImpl;
}

pub struct VehicleProducer {}

impl Producer for VehicleProducer {
    type Interface = VehicleInterface;
    type OfferedProducer = VehicleOfferedProducer;

    fn offer(self) -> com_api::Result<Self::OfferedProducer> {
        todo!()
    }
}

pub struct VehicleOfferedProducer {
    pub left_tire: com_api_runtime_mock::Publisher<Tire>,
    pub exhaust: com_api_runtime_mock::Publisher<Exhaust>,
}

impl OfferedProducer for VehicleOfferedProducer {
    type Interface = VehicleInterface;
    type Producer = VehicleProducer;

    fn unoffer(self) -> Self::Producer {
        VehicleProducer {}
    }
}

pub struct VehicleConsumer {
    pub left_tire: com_api_runtime_mock::SubscribableImpl<Tire>,
    pub exhaust: com_api_runtime_mock::SubscribableImpl<Exhaust>,
}

impl Consumer for VehicleConsumer {}

impl TryFrom<SampleConsumerBuilder> for VehicleConsumer {
    type Error = com_api::Error;
    fn try_from(_builder: SampleConsumerBuilder) -> Result<Self> {
        todo!("VehicleConsumer::try_from")
    }
}

impl TryFrom<SampleProducerBuilder> for VehicleProducer {
    type Error = com_api::Error;
    fn try_from(_builder: SampleProducerBuilder) -> Result<Self> {
        todo!("VehicleProducer::try_from")
    }
}
