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

//! This crate provides the COM API, which is a common interface for different implementations
//! of the COM API, e.g., for different IPC backends.
//! The actual implementations are provided by the `com-api-runtime-mock` and `com-api-runtime-lola` crates.
//! The user must enable one of these features to use the COM API.


pub use com_api_concept::{
    Builder, Consumer, ConsumerBuilder, ConsumerDescriptor, InstanceSpecifier, Interface,
    OfferedProducer, Producer, ProducerBuilder, Reloc, Result, SampleContainer, SampleMaybeUninit,
    SampleMut, ServiceDiscovery, Subscriber, Subscription,
};
