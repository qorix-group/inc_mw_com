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

//! This crate provides a mock implementation of the COM API for testing purposes.
//! It is meant to be used in conjunction with the `com-api` crate.
//! The mock implementation does not perform any real IPC and is not meant to be used in production.
//! It is only meant to be used for testing and development.

#![allow(dead_code)]

use com_api_concept::Reloc;
use core::cmp::Ordering;
use core::fmt::Debug;
use core::future::Future;
use core::marker::PhantomData;
use core::mem::MaybeUninit;
use core::ops::{Deref, DerefMut};
use core::sync::atomic::AtomicUsize;
use std::collections::VecDeque;
use std::path::Path;
use iceoryx2::prelude::*;
use std::sync::Arc;

use com_api_concept::{
    Builder, Consumer, ConsumerBuilder, ConsumerDescriptor, FindServiceSpecifier,
    InstanceSpecifier, Interface, Producer, ProducerBuilder, Result, Runtime, SampleContainer,
    ServiceDiscovery, Subscriber, Subscription,
};

pub struct MockRuntimeImpl {
    node: Arc<Node<ipc_threadsafe::Service>>,
}

// Note: ProviderInfo is currently unused but will be utilized
// with the Producer::offer() method in future implementations.
#[derive(Clone)]
pub struct MockProviderInfo {
    instance_specifier: InstanceSpecifier,
    node: Arc<Node<ipc_threadsafe::Service>>,
}

#[derive(Clone)]
pub struct MockConsumerInfo {
    instance_specifier: InstanceSpecifier,
    node: Arc<Node<ipc_threadsafe::Service>>,
}

impl Runtime for MockRuntimeImpl {
    type ServiceDiscovery<I: Interface + Debug> = SampleConsumerDiscovery<I>;
    type Subscriber<T: Reloc + Send + Debug + 'static> = SubscribableImpl<T>;
    type ProducerBuilder<I: Interface, P: Producer<Self, Interface = I>> = SampleProducerBuilder<I>;
    type Publisher<T: Reloc + Send + Debug + 'static> = Publisher<T>;
    type ProviderInfo = MockProviderInfo;
    type ConsumerInfo = MockConsumerInfo;

    fn find_service<I: Interface + Debug>(
        &self,
        _instance_specifier: FindServiceSpecifier,
    ) -> Self::ServiceDiscovery<I> {
        let specifier: String;
        match _instance_specifier {
            FindServiceSpecifier::Specific(s) => {
                specifier = s.as_ref().to_string();
                println!("Finding service for instance specifier: {}", specifier);
            }
            FindServiceSpecifier::Any => {
                specifier = "any".to_string();
                println!("Finding service for any available instance");
            }
        }
        println!("Creating SampleConsumerDiscovery for specifier: {}", specifier);
        SampleConsumerDiscovery {
            instance_specifier: InstanceSpecifier::new(specifier.as_str()).unwrap(),
            _interface: PhantomData,
            node: Arc::clone(&self.node),
        }
    }

    fn producer_builder<I: Interface, P: Producer<Self, Interface = I>>(
        &self,
        instance_specifier: InstanceSpecifier,
    ) -> Self::ProducerBuilder<I, P> {
        SampleProducerBuilder::new(self, instance_specifier)
    }
}

#[derive(Debug)]
struct MockEvent<T> {
    event: PhantomData<T>,
}

#[derive(Debug)]
struct MockBinding<'a, T>
where
    T: Send + Reloc + Debug + ZeroCopySend,
{
    data: iceoryx2::sample::Sample<ipc_threadsafe::Service, T, ()>,
    event: &'a MockEvent<T>,
}

unsafe impl<'a, T> Send for MockBinding<'a, T> where T: Send + Reloc + Debug + ZeroCopySend
{}

#[derive(Debug)]
enum SampleBinding<'a, T>
where
    T: Send + Reloc + Debug + ZeroCopySend,
{
    Mock(MockBinding<'a, T>),
    Test(Box<T>),
}

#[derive(Debug)]
pub struct Sample<'a, T>
where
    T: Reloc + Send + Debug + ZeroCopySend,
{
    id: usize,
    inner: SampleBinding<'a, T>,
}

static ID_COUNTER: AtomicUsize = AtomicUsize::new(0);

impl<'a, T> From<T> for Sample<'a, T>
where
    T: Reloc + Send + Debug + ZeroCopySend,
{
    fn from(value: T) -> Self {
        Self {
            id: ID_COUNTER.fetch_add(1, core::sync::atomic::Ordering::Relaxed),
            inner: SampleBinding::Test(Box::new(value)),
        }
    }
}

impl<'a, T> Deref for Sample<'a, T>
where
    T: Reloc + Send + Debug + ZeroCopySend,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        match &self.inner {
            SampleBinding::Mock(_mock) => _mock.data.payload(),
            SampleBinding::Test(test) => test.as_ref(),
        }
    }
}

impl<'a, T> com_api_concept::Sample<T> for Sample<'a, T> where T: Send + Reloc + Debug + ZeroCopySend {}

impl<'a, T> PartialEq for Sample<'a, T>
where
    T: Send + Reloc + Debug + ZeroCopySend,
{
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl<'a, T> Eq for Sample<'a, T> where T: Send + Reloc + Debug + ZeroCopySend {}

impl<'a, T> PartialOrd for Sample<'a, T>
where
    T: Send + Reloc + Debug + ZeroCopySend,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<'a, T> Ord for Sample<'a, T>
where
    T: Send + Reloc + Debug + ZeroCopySend,
{
    fn cmp(&self, other: &Self) -> Ordering {
        self.id.cmp(&other.id)
    }
}

#[derive(Debug)]
pub struct SampleMut<'a, T>
where
    T: Reloc + Debug + ZeroCopySend,
{
    data: iceoryx2::sample_mut::SampleMut<ipc_threadsafe::Service, T, ()>,
    lifetime: PhantomData<&'a T>,
}

impl<'a, T> com_api_concept::SampleMut<T> for SampleMut<'a, T>
where
    T: Reloc + Send + Debug + ZeroCopySend,
{
    type Sample = Sample<'a, T>;

    fn into_sample(self) -> Self::Sample {
        todo!()
    }

    fn send(self) -> com_api_concept::Result<()> {
        let return_value = self.data.send();
        match return_value {
            Err(e) => panic!("Failed to send data {e}"),
            Ok(_subscriber_ok) => Ok(()),
        }
    }
}

impl<'a, T> Deref for SampleMut<'a, T>
where
    T: Reloc + Debug + ZeroCopySend,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<'a, T> DerefMut for SampleMut<'a, T>
where
    T: Reloc + Debug + ZeroCopySend,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}


pub struct SampleMaybeUninit<'a, T>
where
    T: Reloc + Send + ZeroCopySend,
{
    data: iceoryx2::sample_mut_uninit::SampleMutUninit<ipc_threadsafe::Service, MaybeUninit<T>, ()>,
    lifetime: PhantomData<&'a T>,
}

impl<'a, T> Debug for SampleMaybeUninit<'a, T>
where
    T: Reloc + Send + ZeroCopySend,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "SampleMaybeUninit")
    }
}

impl<'a, T> com_api_concept::SampleMaybeUninit<T> for SampleMaybeUninit<'a, T>
where
    T: Reloc + Send + Debug + ZeroCopySend,
{
    type SampleMut = SampleMut<'a, T>;

    fn write(self, val: T) -> SampleMut<'a, T> {
        SampleMut {
            data: self.data.write_payload(val),
            lifetime: PhantomData,
        }
    }

    unsafe fn assume_init(self) -> SampleMut<'a, T> {
        SampleMut {
            data: unsafe { self.data.assume_init() },
            lifetime: PhantomData,
        }
    }
}

impl<'a, T> AsMut<core::mem::MaybeUninit<T>> for SampleMaybeUninit<'a, T>
where
    T: Reloc + Send + Debug,
{
    fn as_mut(&mut self) -> &mut core::mem::MaybeUninit<T> {
        todo!()
        //&mut self.data
    }
}

pub struct SubscribableImpl<T: Debug + ZeroCopySend> {
    identifier: String,
    instance_info: Option<MockConsumerInfo>,
    data: PhantomData<T>,
    service: Option<Arc<
        iceoryx2::service::port_factory::publish_subscribe::PortFactory<
            ipc_threadsafe::Service,
            T,
            (),
        >,
    >>,
}

impl<T: Debug + ZeroCopySend> Default for SubscribableImpl<T> {
    fn default() -> Self {
        Self {
            identifier: String::new(),
            instance_info: None,
            data: PhantomData,
            service: None,
        }
    }
}

impl<T: Reloc + Send + Debug + ZeroCopySend + 'static> Subscriber<T, MockRuntimeImpl>
    for SubscribableImpl<T>
{
    type Subscription = SubscriberImpl<T>;
    fn new(identifier: &str, instance_info: MockConsumerInfo) -> com_api_concept::Result<Self> {
        let specifier = instance_info.instance_specifier.as_ref();
        let service_name = format!("{}/{}", specifier, identifier);
        println!("Creating subscriber for service name: {}", service_name);
        let service = instance_info
            .node
            .service_builder(&ServiceName::new(service_name.as_str()).unwrap())
            .publish_subscribe::<T>()
            .open()
            .unwrap();
        Ok(Self {
            identifier: identifier.to_string(),
            instance_info: Some(instance_info),
            data: PhantomData,
            service: Some(Arc::new(service)),
        })
    }
    fn subscribe(&self, _max_num_samples: usize) -> com_api_concept::Result<Self::Subscription> {
        println!(
            "Subscribing to service: {}",
            self.identifier.as_str()
        );
        match &self.service {
            None => return Err(com_api_concept::Error::SubscribeFailed),
            Some(service) => return Ok(SubscriberImpl::new(Arc::clone(service))),
        }
    }
}

pub struct SubscriberImpl<T: Debug + ZeroCopySend + 'static>
where
    T: Reloc + Send + Debug + ZeroCopySend,
{
    data: VecDeque<T>,
    subscriber: iceoryx2::port::subscriber::Subscriber<ipc_threadsafe::Service, T, ()>,
}

impl<T> SubscriberImpl<T>
where
    T: Reloc + Send + Debug + ZeroCopySend,
{
    pub fn new(
        service: Arc<iceoryx2::service::port_factory::publish_subscribe::PortFactory<
            ipc_threadsafe::Service,
            T,
            (),
        >>,
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

impl<T> Subscription<T, MockRuntimeImpl> for SubscriberImpl<T>
where
    T: Reloc + Send + Debug + ZeroCopySend,
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
                        inner: SampleBinding::Mock(MockBinding {
                            data: sample,
                            event: &MockEvent { event: PhantomData },
                        }),
                    });
                    return Ok(1);
                }
                None => return Ok(0),
            },
            Err(_e) => Err(com_api_concept::Error::Fail),
        }
    }

    #[allow(clippy::manual_async_fn)]
    fn receive<'a>(
        &'a self,
        _scratch: &'_ mut SampleContainer<Self::Sample<'a>>,
        _new_samples: usize,
        _max_samples: usize,
    ) -> impl Future<Output = com_api_concept::Result<usize>> + Send {
        async { todo!() }
    }
}

pub struct Publisher<T: Debug + ZeroCopySend + 'static> {
    _data: PhantomData<T>,
    publisher: iceoryx2::port::publisher::Publisher<ipc_threadsafe::Service, T, ()>,
}

// impl<T: Debug + ZeroCopySend> Default for Publisher<T>
// where
//     T: Reloc + Send,
// {
//     fn default() -> Self {
//         Self::new()
//     }
// }

impl<T: Debug + ZeroCopySend> Publisher<T> where T: Reloc + Send {}

impl<T> com_api_concept::Publisher<T, MockRuntimeImpl> for Publisher<T>
where
    T: Reloc + Send + Debug,
{
    type SampleMaybeUninit<'a>
        = SampleMaybeUninit<'a, T>
    where
        Self: 'a;

    fn new(identifier: &str, instance_info: MockProviderInfo) -> com_api_concept::Result<Self>  {
        let specifier = instance_info.instance_specifier.as_ref();
        let service_name = format!("{}/{}", specifier, identifier);
        println!("Creating publisher for service name: {}", service_name);
        let service = instance_info
            .node
            .service_builder(&ServiceName::new(service_name.as_str()).unwrap())
            .publish_subscribe::<T>()
            .create()
            .unwrap()
            .publisher_builder()
            .create()
            .unwrap();
        Ok(Self {
            _data: PhantomData,
            publisher: service,
        })
    }

    fn allocate<'a>(&'a self) -> com_api_concept::Result<Self::SampleMaybeUninit<'a>> {
        let data_result = self.publisher.loan_uninit();
        match data_result {
            Err(_e) => return Err(com_api_concept::Error::AllocateFailed),
            Ok(data_result_ok) => {
                return Ok(SampleMaybeUninit {
                    data: data_result_ok,
                    lifetime: PhantomData,
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
    fn new(_runtime: &MockRuntimeImpl, _instance_specifier: InstanceSpecifier) -> Self {
        Self {
            instance_specifier: _instance_specifier,
            _interface: PhantomData,
            node: Arc::clone(&_runtime.node),
        }
    }
}

impl<I: Interface> ServiceDiscovery<I, MockRuntimeImpl> for SampleConsumerDiscovery<I>
where
    SampleConsumerBuilder<I>: ConsumerBuilder<I, MockRuntimeImpl>,
{
    type ConsumerBuilder = SampleConsumerBuilder<I>;
    type ServiceEnumerator = Vec<SampleConsumerBuilder<I>>;

    fn get_available_instances(&self) -> com_api_concept::Result<Self::ServiceEnumerator> {
        let mut result: Vec<SampleConsumerBuilder<I>> = Vec::new();
        match self.instance_specifier.as_ref() {
            "any" => {
                // Functionality in progress
                let instance_specifier =
                    InstanceSpecifier::new("My/Funk/ServiceName").unwrap();
                result.push(SampleConsumerBuilder {
                    instance_specifier: instance_specifier,
                    _interface: PhantomData,
                    node: Arc::clone(&self.node),
                });
                Ok(result)
            }
            _ => {
                let instance_specifier =
                    InstanceSpecifier::new(self.instance_specifier.as_ref().to_string()).unwrap();
                result.push(SampleConsumerBuilder {
                    instance_specifier: instance_specifier,
                    _interface: PhantomData,
                    node: Arc::clone(&self.node),
                });
                Ok(result)
            }
        }
        
    }

    #[allow(clippy::manual_async_fn)]
    fn get_available_instances_async(
        &self,
    ) -> impl Future<Output = com_api_concept::Result<Self::ServiceEnumerator>> {
        async { Ok(Vec::new()) }
    }
}

pub struct SampleProducerBuilder<I: Interface> {
    instance_specifier: InstanceSpecifier,
    _interface: PhantomData<I>,
    pub node: Arc<Node<ipc_threadsafe::Service>>,
}

impl<I: Interface> SampleProducerBuilder<I> {
    fn new(_runtime: &MockRuntimeImpl, instance_specifier: InstanceSpecifier) -> Self {
        Self {
            instance_specifier,
            _interface: PhantomData,
            node: Arc::clone(&_runtime.node),
        }
    }
}

impl<I: Interface, P: Producer<MockRuntimeImpl, Interface = I>>
    ProducerBuilder<I, P, MockRuntimeImpl> for SampleProducerBuilder<I>
{
}

impl<I: Interface, P: Producer<MockRuntimeImpl, Interface = I>> Builder<P>
    for SampleProducerBuilder<I>
{
    fn build(self) -> Result<P> {
        let instance_info = MockProviderInfo {
            instance_specifier: self.instance_specifier.clone(),
            node: Arc::clone(&self.node),
        };
        Ok(P::new(instance_info).unwrap())
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
    instance_specifier: InstanceSpecifier,
    _interface: PhantomData<I>,
    pub node: Arc<Node<ipc_threadsafe::Service>>,
}

impl<I: Interface> ConsumerDescriptor<MockRuntimeImpl> for SampleConsumerBuilder<I> {
    fn get_instance_identifier(&self) -> String {
        println!("Getting instance identifier: {}", self.instance_specifier.as_ref().to_string());
        return self.instance_specifier.as_ref().to_string();
    }
}

impl<I: Interface + Debug> ConsumerBuilder<I, MockRuntimeImpl>
    for SampleConsumerBuilder<I>
{
}

impl<I: Interface + Debug> Builder<I::Consumer<MockRuntimeImpl>>
    for SampleConsumerBuilder<I>
{
    fn build(self) -> com_api_concept::Result<I::Consumer<MockRuntimeImpl>> {
        let instance_info = MockConsumerInfo {
            instance_specifier: self.instance_specifier.clone(),
            node: Arc::clone(&self.node),
        };

        Ok(Consumer::new(instance_info)?)
    }
}

pub struct RuntimeBuilderImpl {}

impl Builder<MockRuntimeImpl> for RuntimeBuilderImpl {
    fn build(self) -> com_api_concept::Result<MockRuntimeImpl> {
        let node = NodeBuilder::new().create::<ipc_threadsafe::Service>();
        match node {
            Ok(n) => Ok(MockRuntimeImpl { node: Arc::new(n) }),
            Err(_e) => Err(com_api_concept::Error::Fail),
        }
    }
}

/// Entry point for the default implementation for the com module of s-core
impl com_api_concept::RuntimeBuilder<MockRuntimeImpl> for RuntimeBuilderImpl {
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
    /// Creates a new instance of the default implementation for the com module of s-core
    pub fn new() -> Self {
        Self {}
    }
}
