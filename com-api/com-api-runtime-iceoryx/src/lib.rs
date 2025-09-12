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

use iceoryx2::prelude::*;
use std::cmp::Ordering;
use std::collections::VecDeque;
use std::marker::PhantomData;
use std::mem::MaybeUninit;
use std::ops::{Deref, DerefMut};
use std::path::Path;
use std::rc::Rc;
use std::sync::atomic::AtomicUsize;

use com_api_concept::{
    AdapterConcept, BuilderConcept, ConsumerBuilderConcept, ConsumerDescriptorConcept,
    InstanceSpecifier, InterfaceConcept, Reloc, SampleConcept, SampleContainer,
    SampleMaybeUninitConcept, SampleMutConcept, ServiceDiscoveryConcept, SubscriberConcept,
    SubscriptionConcept,
};

pub struct IceoryxAdapter {
    node: Rc<Node<ipc::Service>>,
}

impl AdapterConcept for IceoryxAdapter {
    type Sample<'a, T: Reloc + Send + 'a + std::fmt::Debug + 'a> = IceoryxSample<'a, T>;
}

impl IceoryxAdapter {
    // TODO: Any chance that these can be moved to a trait so that this becomes more testable?
    // If yes, this trait is certainly located here since
    pub fn find_service<I: InterfaceConcept>(
        &self,
        _instance_specifier: InstanceSpecifier,
    ) -> IceoryxConsumerDiscovery<I> {
        IceoryxConsumerDiscovery {
            instance_specifier: _instance_specifier,
            _interface: PhantomData,
            node: Rc::clone(&self.node),
        }
    }

    pub fn producer_builder<I: InterfaceConcept + ZeroCopySend + std::fmt::Debug>(
        &self,
        instance_specifier: InstanceSpecifier,
    ) -> IceoryxProducerBuilder<I> {
        IceoryxProducerBuilder::new(self, instance_specifier)
    }
}

#[derive(Debug)]
struct IceoryxEvent<T> {
    event: PhantomData<T>,
}

#[derive(Debug)]
struct IceoryxBinding<'a, T>
where
    T: Send + Reloc + std::fmt::Debug + ZeroCopySend,
{
    data: iceoryx2::sample::Sample<ipc::Service, T, ()>,
    event: &'a IceoryxEvent<T>,
}

unsafe impl<'a, T> Send for IceoryxBinding<'a, T> where
    T: Send + Reloc + std::fmt::Debug + ZeroCopySend
{
}

#[derive(Debug)]
enum SampleBinding<'a, T>
where
    T: Send + Reloc + std::fmt::Debug + ZeroCopySend,
{
    Iceoryx(IceoryxBinding<'a, T>),
    Test(Box<T>),
}

#[derive(Debug)]
pub struct IceoryxSample<'a, T>
where
    T: Reloc + Send + std::fmt::Debug + ZeroCopySend,
{
    id: usize,
    inner: SampleBinding<'a, T>,
}

unsafe impl<T> ZeroCopySend for IceoryxSample<'_, T> where
    T: Reloc + Send + std::fmt::Debug + ZeroCopySend
{
}

static ID_COUNTER: AtomicUsize = AtomicUsize::new(0);

impl<'a, T> From<T> for IceoryxSample<'a, T>
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

impl<'a, T> Deref for IceoryxSample<'a, T>
where
    T: Reloc + Send + std::fmt::Debug + ZeroCopySend,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        match &self.inner {
            SampleBinding::Iceoryx(_iceoryx) => _iceoryx.data.payload(),
            SampleBinding::Test(test) => test.as_ref(),
        }
    }
}

impl<'a, T> SampleConcept<T> for IceoryxSample<'a, T> where
    T: Send + Reloc + std::fmt::Debug + ZeroCopySend
{
}

impl<'a, T> PartialEq for IceoryxSample<'a, T>
where
    T: Send + Reloc + std::fmt::Debug + ZeroCopySend,
{
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl<'a, T> Eq for IceoryxSample<'a, T> where T: Send + Reloc + std::fmt::Debug + ZeroCopySend {}

impl<'a, T> PartialOrd for IceoryxSample<'a, T>
where
    T: Send + Reloc + std::fmt::Debug + ZeroCopySend,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<'a, T> Ord for IceoryxSample<'a, T>
where
    T: Send + Reloc + std::fmt::Debug + ZeroCopySend,
{
    fn cmp(&self, other: &Self) -> Ordering {
        self.id.cmp(&other.id)
    }
}

pub struct IceoryxSampleMut<'a, T>
where
    T: Reloc + std::fmt::Debug + ZeroCopySend,
{
    data: iceoryx2::sample_mut::SampleMut<ipc::Service, T, ()>,
    _lifetime: PhantomData<&'a T>,
}

impl<'a, T> SampleMutConcept<T> for IceoryxSampleMut<'a, T>
where
    T: Reloc + Send + std::fmt::Debug + ZeroCopySend,
{
    type Sample = IceoryxSample<'a, T>;

    fn into_sample(self) -> Self::Sample {
        todo!()
    }

    fn send(self) -> com_api_concept::Result<()> {
        let _ = self.data.send();
        Ok(())
    }
}

impl<'a, T> Deref for IceoryxSampleMut<'a, T>
where
    T: Reloc + std::fmt::Debug + ZeroCopySend,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<'a, T> DerefMut for IceoryxSampleMut<'a, T>
where
    T: Reloc + std::fmt::Debug + ZeroCopySend,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}

pub struct IceoryxSampleMaybeUninit<'a, T: ZeroCopySend>
where
    T: Reloc + Send + ZeroCopySend,
{
    data: iceoryx2::sample_mut_uninit::SampleMutUninit<ipc::Service, MaybeUninit<T>, ()>,
    _lifetime: PhantomData<&'a T>,
}

impl<'a, T> SampleMaybeUninitConcept<T> for IceoryxSampleMaybeUninit<'a, T>
where
    T: Reloc + Send + ZeroCopySend + std::fmt::Debug,
{
    type SampleMut = IceoryxSampleMut<'a, T>;

    fn write(self, val: T) -> IceoryxSampleMut<'a, T> {
        IceoryxSampleMut {
            data: self.data.write_payload(val),
            _lifetime: PhantomData,
        }
    }

    // unsafe fn assume_init(self) -> SampleMut<'a, T> {
    //     IceoryxSampleMut {
    //         data: unsafe { self.data.assume_init() },
    //         _lifetime: PhantomData,
    //     }
    // }
}

pub struct IceoryxSubscribable<T: std::fmt::Debug + ZeroCopySend> {
    _data: PhantomData<T>,
    service: Option<
        iceoryx2::service::port_factory::publish_subscribe::PortFactory<ipc::Service, T, ()>,
    >,
}

impl<T: std::fmt::Debug + ZeroCopySend> Default for IceoryxSubscribable<T> {
    fn default() -> Self {
        Self {
            _data: PhantomData,
            service: None,
        }
    }
}

impl<T: std::fmt::Debug + ZeroCopySend> IceoryxSubscribable<T> {
    pub fn new(
        service: iceoryx2::service::port_factory::publish_subscribe::PortFactory<
            ipc::Service,
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

impl<T: Reloc + Send + ZeroCopySend + std::fmt::Debug + 'static> SubscriberConcept<T>
    for IceoryxSubscribable<T>
{
    type Subscription = IceoryxSubscriber<T>;

    fn subscribe(self, _max_num_samples: usize) -> com_api_concept::Result<Self::Subscription> {
        match self.service {
            None => return Err(com_api_concept::Error::SubscribeFailed),
            Some(service) => return Ok(IceoryxSubscriber::new(service)),
        }
    }
}

pub struct IceoryxSubscriber<T: std::fmt::Debug + ZeroCopySend + 'static>
where
    T: Reloc + Send,
{
    data: VecDeque<T>,
    subscriber: iceoryx2::port::subscriber::Subscriber<ipc::Service, T, ()>,
}

impl<T: std::fmt::Debug + ZeroCopySend> IceoryxSubscriber<T>
where
    T: Reloc + Send,
{
    pub fn new(
        service: iceoryx2::service::port_factory::publish_subscribe::PortFactory<
            ipc::Service,
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

impl<T> SubscriptionConcept<T> for IceoryxSubscriber<T>
where
    T: Reloc + Send + ZeroCopySend + std::fmt::Debug,
{
    type Subscriber = IceoryxSubscribable<T>;
    type Sample<'a>
        = IceoryxSample<'a, T>
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
                    let _res = _scratch.push_back(IceoryxSample {
                        id: ID_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
                        inner: SampleBinding::Iceoryx(IceoryxBinding {
                            data: sample,
                            event: &IceoryxEvent { event: PhantomData },
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

pub struct IceoryxPublisher<T: std::fmt::Debug + ZeroCopySend + 'static> {
    _data: PhantomData<T>,
    publisher: iceoryx2::port::publisher::Publisher<ipc::Service, T, ()>,
}

// impl<T: std::fmt::Debug + ZeroCopySend> Default for IceoryxPublisher<T>
// where
//     T: Reloc + Send,
// {
//     fn default() -> Self {
//         Self::new()
//     }
// }

impl<T: std::fmt::Debug + ZeroCopySend> IceoryxPublisher<T>
where
    T: Reloc + Send,
{
    pub fn new(
        service: iceoryx2::service::port_factory::publish_subscribe::PortFactory<
            ipc::Service,
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

    pub fn allocate<'a>(&'a self) -> com_api_concept::Result<IceoryxSampleMaybeUninit<'a, T>> {
        let data_result = self.publisher.loan_uninit();
        match data_result {
            Err(_e) => return Err(com_api_concept::Error::AllocateFailed),
            Ok(data_result_ok) => {
                return Ok(IceoryxSampleMaybeUninit {
                    data: data_result_ok,
                    _lifetime: PhantomData,
                });
            }
        }
    }
}

pub struct IceoryxConsumerDiscovery<I> {
    pub instance_specifier: InstanceSpecifier,
    _interface: PhantomData<I>,
    node: Rc<Node<ipc::Service>>,
}

impl<I> IceoryxConsumerDiscovery<I> {
    fn new(_runtime: &IceoryxAdapter, _instance_specifier: InstanceSpecifier) -> Self {
        Self {
            instance_specifier: _instance_specifier,
            _interface: PhantomData,
            node: Rc::clone(&_runtime.node),
        }
    }
}

impl<I: InterfaceConcept> ServiceDiscoveryConcept<I, IceoryxAdapter> for IceoryxConsumerDiscovery<I>
where
    IceoryxConsumerBuilder<I>: ConsumerBuilderConcept<I, IceoryxAdapter>,
{
    type ConsumerBuilder = IceoryxConsumerBuilder<I>;
    type ServiceEnumerator = Vec<IceoryxConsumerBuilder<I>>;

    fn get_available_instances(&self) -> com_api_concept::Result<Self::ServiceEnumerator> {
        let mut result: Vec<IceoryxConsumerBuilder<I>> = Vec::new();
        let instance_specifier = InstanceSpecifier {
            specifier: self.instance_specifier.specifier.clone(),
        };
        result.push(IceoryxConsumerBuilder {
            instance_specifier: instance_specifier,
            _interface: PhantomData,
            node: Rc::clone(&self.node),
        });
        Ok(result)
    }
}

pub struct IceoryxProducerBuilder<I: InterfaceConcept + std::fmt::Debug + ZeroCopySend> {
    pub instance_specifier: InstanceSpecifier,
    _interface: PhantomData<I>,
    pub node: Rc<Node<ipc::Service>>,
}

impl<I: InterfaceConcept + std::fmt::Debug + ZeroCopySend> IceoryxProducerBuilder<I> {
    fn new(_runtime: &IceoryxAdapter, instance_specifier: InstanceSpecifier) -> Self {
        Self {
            instance_specifier,
            _interface: PhantomData,
            node: Rc::clone(&_runtime.node),
        }
    }
}

pub struct IceoryxConsumerDescriptor<I: InterfaceConcept> {
    _interface: PhantomData<I>,
}

impl<I: InterfaceConcept> Clone for IceoryxConsumerDescriptor<I> {
    fn clone(&self) -> Self {
        Self {
            _interface: PhantomData,
        }
    }
}

pub struct IceoryxConsumerBuilder<I: InterfaceConcept> {
    pub instance_specifier: InstanceSpecifier,
    _interface: PhantomData<I>,
    pub node: Rc<Node<ipc::Service>>,
}

impl<I: InterfaceConcept> ConsumerDescriptorConcept<IceoryxAdapter> for IceoryxConsumerBuilder<I> {
    fn get_instance_id(&self) -> usize {
        42
    }
}

pub struct IceoryxAdapterBuilder {}

impl BuilderConcept<IceoryxAdapter> for IceoryxAdapterBuilder {
    fn build(self) -> com_api_concept::Result<IceoryxAdapter> {
        let node = NodeBuilder::new().create::<ipc::Service>();
        match node {
            Ok(n) => Ok(IceoryxAdapter { node: Rc::new(n) }),
            Err(_e) => Err(com_api_concept::Error::Fail),
        }
    }
}

/// Entry point for the default implementation for the com module of s-core
impl com_api_concept::AdapterBuilderConcept<IceoryxAdapter> for IceoryxAdapterBuilder {
    fn load_config(&mut self, _config: &Path) -> &mut Self {
        self
    }
}

impl Default for IceoryxAdapterBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl IceoryxAdapterBuilder {
    /// Creates a new instance of the default implementation of the com layer
    pub fn new() -> Self {
        Self {}
    }
}

#[cfg(test)]
mod test {
    use com_api_concept::{SampleContainer, SubscriptionConcept};

    #[test]
    fn receive_stuff() {
        let test_subscriber = super::IceoryxSubscriber::<u32>::new();
        for _ in 0..10 {
            let mut sample_buf = SampleContainer::new();
            let receive_result = test_subscriber.try_receive(&mut sample_buf, 1);
            match receive_result {
                Ok(0) => panic!("No sample received"),
                Ok(x) => {
                    println!(
                        "{} samples received: sample[0] = {}",
                        x,
                        *sample_buf.front().unwrap()
                    )
                }
                Err(e) => panic!("{:?}", e),
            }
        }
    }

    #[test]
    fn receive_async_stuff() {
        let test_subscriber = super::IceoryxSubscriber::<u32>::new();
        // block on an asynchronous reception of data from test_subscriber
        futures::executor::block_on(async {
            let mut sample_buf = SampleContainer::new();
            match test_subscriber.receive(&mut sample_buf, 1, 1).await {
                Ok(0) => panic!("No sample received"),
                Ok(x) => {
                    println!(
                        "{} samples received: sample[0] = {}",
                        x,
                        *sample_buf.front().unwrap()
                    )
                }
                Err(e) => panic!("{:?}", e),
            }
        })
    }
}
