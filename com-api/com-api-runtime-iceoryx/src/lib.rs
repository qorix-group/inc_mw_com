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

#![allow(dead_code)]

use std::cmp::Ordering;
use std::collections::VecDeque;
use std::marker::PhantomData;
use std::mem::MaybeUninit;
use std::ops::{Deref, DerefMut};
use std::path::Path;

use std::sync::Arc;
use std::sync::atomic::AtomicUsize;

use iceoryx2::prelude::*;

use com_api_concept::{
    Builder, ConsumerBuilder, ConsumerDescriptor, InstanceSpecifier, Interface, Reloc, Runtime,
    SampleContainer, ServiceDiscovery, Subscriber, Subscription,
};

pub struct RuntimeImpl {
    node: Arc<Node<ipc_threadsafe::Service>>,
}

impl Runtime for RuntimeImpl {
    type Sample<'a, T: Reloc + Send + std::fmt::Debug + 'a> = Sample<'a, T>;
}

impl RuntimeImpl {
    // TODO: Any chance that these can be moved to a trait so that this becomes more testable?
    // If yes, this trait is certainly located here since
    pub fn find_service<I: Interface>(
        &self,
        _instance_specifier: InstanceSpecifier,
    ) -> SampleConsumerDiscovery<I> {
        SampleConsumerDiscovery {
            instance_specifier: _instance_specifier,
            _interface: PhantomData,
            node: Arc::clone(&self.node),
        }
    }

    pub fn producer_builder<I: Interface + std::fmt::Debug>(
        &self,
        instance_specifier: InstanceSpecifier,
    ) -> SampleProducerBuilder<I> {
        SampleProducerBuilder::new(self, instance_specifier)
    }
}

#[derive(Debug)]
struct LolaEvent<T> {
    event: PhantomData<T>,
}

#[derive(Debug)]
struct LolaBinding<'a, T>
where
    T: Send,
{
    data: *mut T,
    event: &'a LolaEvent<T>,
}

unsafe impl<'a, T> Send for LolaBinding<'a, T> where T: Send {}

#[derive(Debug)]
struct Iceoryx2Event<T> {
    event: PhantomData<T>,
}

#[derive(Debug)]
struct Iceoryx2Binding<'a, T>
where
    T: Send + Reloc + std::fmt::Debug + ZeroCopySend,
{
    data: iceoryx2::sample::Sample<ipc_threadsafe::Service, T, ()>,
    event: &'a Iceoryx2Event<T>,
}

unsafe impl<'a, T> Send for Iceoryx2Binding<'a, T> where
    T: Send + Reloc + std::fmt::Debug + ZeroCopySend
{
}

#[derive(Debug)]
enum SampleBinding<'a, T>
where
    T: Send + Reloc + std::fmt::Debug + ZeroCopySend,
{
    Iceoryx2(Iceoryx2Binding<'a, T>),
    Lola(LolaBinding<'a, T>),
    Test(Box<T>),
}

#[derive(Debug)]
pub struct Sample<'a, T>
where
    T: Reloc + Send + std::fmt::Debug + ZeroCopySend,
{
    id: usize,
    inner: SampleBinding<'a, T>,
}

unsafe impl<T> ZeroCopySend for Sample<'_, T> where T: Reloc + Send + std::fmt::Debug + ZeroCopySend {}

static ID_COUNTER: AtomicUsize = AtomicUsize::new(0);

impl<'a, T> From<T> for Sample<'a, T>
where
    T: Reloc + Send + std::fmt::Debug + ZeroCopySend,
{
    fn from(value: T) -> Self {
        Self {
            id: ID_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
            inner: SampleBinding::Test(Box::new(value)),
        }
    }
}

impl<'a, T> Deref for Sample<'a, T>
where
    T: Reloc + Send + std::fmt::Debug + ZeroCopySend,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        match &self.inner {
            SampleBinding::Lola(_lola) => unimplemented!(),
            SampleBinding::Iceoryx2(_iceoryx2) => _iceoryx2.data.payload(),
            SampleBinding::Test(test) => test.as_ref(),
        }
    }
}

impl<'a, T> com_api_concept::Sample<T> for Sample<'a, T> where
    T: Send + Reloc + std::fmt::Debug + ZeroCopySend
{
}

impl<'a, T> PartialEq for Sample<'a, T>
where
    T: Send + Reloc + std::fmt::Debug + ZeroCopySend,
{
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl<'a, T> Eq for Sample<'a, T> where T: Send + Reloc + std::fmt::Debug + ZeroCopySend {}

impl<'a, T> PartialOrd for Sample<'a, T>
where
    T: Send + Reloc + std::fmt::Debug + ZeroCopySend,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<'a, T> Ord for Sample<'a, T>
where
    T: Send + Reloc + std::fmt::Debug + ZeroCopySend,
{
    fn cmp(&self, other: &Self) -> Ordering {
        self.id.cmp(&other.id)
    }
}

pub struct SampleMut<'a, T>
where
    T: Reloc + std::fmt::Debug + iceoryx2::prelude::ZeroCopySend,
{
    data: iceoryx2::sample_mut::SampleMut<ipc_threadsafe::Service, T, ()>,
    _lifetime: PhantomData<&'a T>,
}

impl<'a, T> com_api_concept::SampleMut<T> for SampleMut<'a, T>
where
    T: Reloc + Send + std::fmt::Debug + iceoryx2::prelude::ZeroCopySend,
{
    type Sample = Sample<'a, T>;

    fn into_sample(self) -> Self::Sample {
        todo!()
    }

    fn send(self) -> com_api_concept::Result<()> {
        let _ = self.data.send();
        Ok(())
    }
}

impl<'a, T> Deref for SampleMut<'a, T>
where
    T: Reloc + std::fmt::Debug + iceoryx2::prelude::ZeroCopySend,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<'a, T> DerefMut for SampleMut<'a, T>
where
    T: Reloc + std::fmt::Debug + iceoryx2::prelude::ZeroCopySend,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}

pub struct SampleMaybeUninit<'a, T: ZeroCopySend>
where
    T: Reloc + Send + ZeroCopySend,
{
    data: iceoryx2::sample_mut_uninit::SampleMutUninit<ipc_threadsafe::Service, MaybeUninit<T>, ()>,
    _lifetime: PhantomData<&'a T>,
}

impl<'a, T> com_api_concept::SampleMaybeUninit<T> for SampleMaybeUninit<'a, T>
where
    T: Reloc + Send + ZeroCopySend + std::fmt::Debug,
{
    type SampleMut = SampleMut<'a, T>;

    fn write(self, val: T) -> SampleMut<'a, T> {
        SampleMut {
            data: self.data.write_payload(val),
            _lifetime: PhantomData,
        }
    }

    // unsafe fn assume_init(self) -> SampleMut<'a, T> {
    //     SampleMut {
    //         data: unsafe { self.data.assume_init() },
    //         _lifetime: PhantomData,
    //     }
    // }
}

pub struct SubscribableImpl<T: std::fmt::Debug + iceoryx2::prelude::ZeroCopySend> {
    _data: PhantomData<T>,
    service: Option<
        iceoryx2::service::port_factory::publish_subscribe::PortFactory<
            ipc_threadsafe::Service,
            T,
            (),
        >,
    >,
}

impl<T: std::fmt::Debug + iceoryx2::prelude::ZeroCopySend> Default for SubscribableImpl<T> {
    fn default() -> Self {
        Self {
            _data: PhantomData,
            service: None,
        }
    }
}

impl<T: std::fmt::Debug + iceoryx2::prelude::ZeroCopySend> SubscribableImpl<T> {
    pub fn new(
        service: iceoryx2::service::port_factory::publish_subscribe::PortFactory<
            ipc_threadsafe::Service,
            T,
            (),
        >,
    ) -> Self {
        Self {
            _data: PhantomData,
            service: Some(service),
        }
    }
}

impl<T: Reloc + Send + iceoryx2::prelude::ZeroCopySend + std::fmt::Debug + 'static> Subscriber<T>
    for SubscribableImpl<T>
{
    type Subscription = SubscriberImpl<T>;

    fn subscribe(self, _max_num_samples: usize) -> com_api_concept::Result<Self::Subscription> {
        match self.service {
            None => return Err(com_api_concept::Error::SubscribeFailed),
            Some(service) => return Ok(SubscriberImpl::new(service)),
        }
    }
}

pub struct SubscriberImpl<T: std::fmt::Debug + iceoryx2::prelude::ZeroCopySend + 'static>
where
    T: Reloc + Send,
{
    data: VecDeque<T>,
    subscriber: iceoryx2::port::subscriber::Subscriber<ipc_threadsafe::Service, T, ()>,
}

impl<T: std::fmt::Debug + iceoryx2::prelude::ZeroCopySend> SubscriberImpl<T>
where
    T: Reloc + Send,
{
    pub fn new(
        service: iceoryx2::service::port_factory::publish_subscribe::PortFactory<
            ipc_threadsafe::Service,
            T,
            (),
        >,
    ) -> Self {
        let subscriber = service.subscriber_builder().create();
        match subscriber {
            Err(e) => panic!("Failed to create subscriber: {e}"),
            Ok(subscriber_ok) => {
                return Self {
                    data: Default::default(),
                    subscriber: subscriber_ok,
                };
            }
        }
    }

    pub fn add_data(&mut self, data: T) {
        self.data.push_front(data);
    }
}

impl<T> Subscription<T> for SubscriberImpl<T>
where
    T: Reloc + Send + iceoryx2::prelude::ZeroCopySend + std::fmt::Debug,
{
    type Subscriber = SubscribableImpl<T>;
    type Sample<'a>
        = Sample<'a, T>
    where
        T: 'a;

    fn unsubscribe(self) -> Self::Subscriber {
        Default::default()
    }

    fn try_receive<'a>(
        &'a self,
        _scratch: &'_ mut SampleContainer<Self::Sample<'a>>,
        _max_samples: usize,
    ) -> com_api_concept::Result<usize> {
        let _result = self.subscriber.receive();
        match _result {
            Ok(option) => match option {
                Some(sample) => {
                    let _res = _scratch.push_back(Sample {
                        id: ID_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
                        inner: SampleBinding::Iceoryx2(Iceoryx2Binding {
                            data: sample,
                            event: &Iceoryx2Event { event: PhantomData },
                        }),
                    });
                    return Ok(1);
                }
                None => return Ok(0),
            },
            Err(_e) => {
                println!("Error receiving sample: {:?}", _e);
                Err(com_api_concept::Error::Fail)
            }
        }
    }

    #[allow(clippy::manual_async_fn)]
    fn receive<'a>(
        &'a self,
        _scratch: &'_ mut SampleContainer<Self::Sample<'a>>,
        _new_samples: usize,
        _max_samples: usize,
    ) -> impl Future<Output = com_api_concept::Result<usize>> {
        async move {
            let received = self.try_receive(_scratch, _max_samples)?;
            if received > 0 || _new_samples == 0 {
                return Ok(received);
            }

            Err(com_api_concept::Error::Timeout)
        }
    }
}

pub struct Publisher<T: std::fmt::Debug + iceoryx2::prelude::ZeroCopySend + 'static> {
    _data: PhantomData<T>,
    publisher: iceoryx2::port::publisher::Publisher<ipc_threadsafe::Service, T, ()>,
}

// impl<T: std::fmt::Debug + iceoryx2::prelude::ZeroCopySend> Default for Publisher<T>
// where
//     T: Reloc + Send,
// {
//     fn default() -> Self {
//         Self::new()
//     }
// }

impl<T: std::fmt::Debug + iceoryx2::prelude::ZeroCopySend> Publisher<T>
where
    T: Reloc + Send,
{
    pub fn new(
        service: iceoryx2::service::port_factory::publish_subscribe::PortFactory<
            ipc_threadsafe::Service,
            T,
            (),
        >,
    ) -> Self {
        let publisher = service.publisher_builder().create();
        match publisher {
            Err(e) => panic!("Failed to create publisher: {e}"),
            Ok(publisher_ok) => {
                return Self {
                    _data: PhantomData,
                    publisher: publisher_ok,
                };
            }
        }
    }

    pub fn allocate<'a>(&'a self) -> com_api_concept::Result<SampleMaybeUninit<'a, T>> {
        let data_result = self.publisher.loan_uninit();
        match data_result {
            Err(_e) => return Err(com_api_concept::Error::AllocateFailed),
            Ok(data_result_ok) => {
                return Ok(SampleMaybeUninit {
                    data: data_result_ok,
                    _lifetime: PhantomData,
                });
            }
        }
    }
}

pub struct SampleConsumerDiscovery<I> {
    pub instance_specifier: InstanceSpecifier,
    _interface: PhantomData<I>,
    node: Arc<Node<ipc_threadsafe::Service>>,
}

impl<I> SampleConsumerDiscovery<I> {
    fn new(_runtime: &RuntimeImpl, _instance_specifier: InstanceSpecifier) -> Self {
        Self {
            instance_specifier: _instance_specifier,
            _interface: PhantomData,
            node: Arc::clone(&_runtime.node),
        }
    }
}

impl<I: Interface> ServiceDiscovery<I, RuntimeImpl> for SampleConsumerDiscovery<I> {
    type ConsumerBuilder = SampleConsumerBuilder<I>;
    type ServiceEnumerator = Vec<SampleConsumerBuilder<I>>;

    fn get_available_instances(&self) -> com_api_concept::Result<Self::ServiceEnumerator> {
        let mut result: Vec<SampleConsumerBuilder<I>> = Vec::new();
        let instance_specifier = InstanceSpecifier {
            specifier: self.instance_specifier.specifier.clone(),
        };
        result.push(SampleConsumerBuilder {
            instance_specifier: instance_specifier,
            _interface: PhantomData,
            node: Arc::clone(&self.node),
        });
        Ok(result)
    }
}

pub struct SampleProducerBuilder<I: Interface + std::fmt::Debug> {
    pub instance_specifier: InstanceSpecifier,
    _interface: PhantomData<I>,
    pub node: Arc<Node<ipc_threadsafe::Service>>,
}

impl<I: Interface + std::fmt::Debug> SampleProducerBuilder<I> {
    fn new(_runtime: &RuntimeImpl, instance_specifier: InstanceSpecifier) -> Self {
        Self {
            instance_specifier,
            _interface: PhantomData,
            node: Arc::clone(&_runtime.node),
        }
    }
}

pub struct SampleConsumerDescriptor<I: Interface> {
    _interface: PhantomData<I>,
}

impl<I: Interface> Clone for SampleConsumerDescriptor<I> {
    fn clone(&self) -> Self {
        Self {
            _interface: PhantomData,
        }
    }
}

pub struct SampleConsumerBuilder<I: Interface> {
    pub instance_specifier: InstanceSpecifier,
    _interface: PhantomData<I>,
    pub node: Arc<Node<ipc_threadsafe::Service>>,
}

impl<I: Interface> ConsumerDescriptor<RuntimeImpl> for SampleConsumerBuilder<I> {
    fn get_instance_id(&self) -> usize {
        42
    }
}

impl<I: Interface> ConsumerBuilder<I, RuntimeImpl> for SampleConsumerBuilder<I> {}

pub struct RuntimeBuilderImpl {}

impl Builder<RuntimeImpl> for RuntimeBuilderImpl {
    fn build(self) -> com_api_concept::Result<RuntimeImpl> {
        let node = NodeBuilder::new().create::<ipc_threadsafe::Service>();
        match node {
            Ok(n) => Ok(RuntimeImpl { node: Arc::new(n) }),
            Err(_e) => Err(com_api_concept::Error::Fail),
        }
    }
}

/// Entry point for the default implementation for the com module of s-core
impl com_api_concept::RuntimeBuilder<RuntimeImpl> for RuntimeBuilderImpl {
    fn load_config(&mut self, _config: &Path) -> &mut Self {
        self
    }
}

impl Default for RuntimeBuilderImpl {
    fn default() -> Self {
        Self::new()
    }
}

impl RuntimeBuilderImpl {
    /// Creates a new instance of the default implementation of the com layer
    pub fn new() -> Self {
        Self {}
    }
}
