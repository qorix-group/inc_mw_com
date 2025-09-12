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

#[cfg(not(any(feature = "iceoryx", feature = "lola")))]
compile_error!("You must enable at least one feature: `iceoryx` or `lola`!");

#[cfg(feature = "iceoryx")]
pub type AdapterBuilder = com_api_runtime_iceoryx::IceoryxAdapterBuilder;
#[cfg(feature = "iceoryx")]
pub type Adapter = com_api_runtime_iceoryx::IceoryxAdapter;
#[cfg(feature = "iceoryx")]
pub type ConsumerBuilder = com_api_runtime_iceoryx::IceoryxConsumerBuilder<Adapter>;
#[cfg(feature = "iceoryx")]
pub type Subscriber = com_api_runtime_iceoryx::IceoryxSubscriber<Adapter>;

#[cfg(feature = "lola")]
pub type AdapterBuilder = com_api_runtime_lola::LolaAdapterBuilder;
#[cfg(feature = "lola")]
pub type Adapter = com_api_runtime_lola::LolaAdapter;
#[cfg(feature = "lola")]
pub type ConsumerBuilder = com_api_runtime_lola::LolaConsumerBuilder<Adapter>;
#[cfg(feature = "lola")]
pub type Subscriber = com_api_runtime_lola::LolaSubscriber<Adapter>;

pub use com_api_concept::{
    BuilderConcept, ConsumerBuilderConcept, ConsumerConcept, ConsumerDescriptorConcept,
    InstanceSpecifier, InterfaceConcept, OfferedProducerConcept, ProducerBuilderConcept,
    ProducerConcept, Reloc, Result, SampleConcept, SampleContainer, SampleMaybeUninitConcept,
    SampleMutConcept, ServiceDiscoveryConcept, SubscriberConcept, SubscriptionConcept,
};
