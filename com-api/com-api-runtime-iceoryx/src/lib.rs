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
use std::sync::atomic::AtomicUsize;

use com_api_concept::{
    BuilderConcept, ConsumerBuilderConcept, ConsumerDescriptorConcept, InstanceSpecifier, InterfaceConcept, Reloc, AdapterConcept,
    SampleContainer, ServiceDiscoveryConcept, SubscriberConcept, SubscriptionConcept, SampleMaybeUninitConcept, SampleConcept, SampleMutConcept
};

pub struct IceoryxAdapter {}

impl AdapterConcept for IceoryxAdapter {
    type Sample<'a, T: Reloc + Send + 'a + std::fmt::Debug> = IceoryxSample<'a, T>;
}

impl IceoryxAdapter {
    // TODO: Any chance that these can be moved to a trait so that this becomes more testable?
    // If yes, this trait is certainly located here since
    pub fn find_service<I: InterfaceConcept>(
        &self,
        _instance_specifier: InstanceSpecifier,
    ) -> IceoryxConsumerDiscovery<I> {
        IceoryxConsumerDiscovery {
            _interface: PhantomData,
        }
    }

    pub fn producer_builder<I: InterfaceConcept>(
        &self,
        instance_specifier: InstanceSpecifier,
    ) -> IceoryxProducerBuilder<I> {
        IceoryxProducerBuilder::new(self, instance_specifier)
    }
}

struct IceoryxEvent<T> {
    event: PhantomData<T>,
}

struct IceoryxBinding<'a, T>
where
    T: Send,
{
    data: *mut T,
    event: &'a IceoryxEvent<T>,
}

unsafe impl<'a, T> Send for IceoryxBinding<'a, T> where T: Send {}

enum SampleBinding<'a, T>
where
    T: Send,
{
    Iceoryx(IceoryxBinding<'a, T>),
    Test(Box<T>),
}

pub struct IceoryxSample<'a, T>
where
    T: Reloc + Send,
{
    id: usize,
    inner: SampleBinding<'a, T>,
}

static ID_COUNTER: AtomicUsize = AtomicUsize::new(0);

impl<'a, T> From<T> for IceoryxSample<'a, T>
where
    T: Reloc + Send,
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
    T: Reloc + Send,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        match &self.inner {
            SampleBinding::Iceoryx(_iceoryx) => unimplemented!(),
            SampleBinding::Test(test) => test.as_ref(),
        }
    }
}

impl<'a, T> SampleConcept<T> for IceoryxSample<'a, T> where T: Send + Reloc {}

impl<'a, T> PartialEq for IceoryxSample<'a, T>
where
    T: Send + Reloc,
{
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl<'a, T> Eq for IceoryxSample<'a, T> where T: Send + Reloc {}

impl<'a, T> PartialOrd for IceoryxSample<'a, T>
where
    T: Send + Reloc,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<'a, T> Ord for IceoryxSample<'a, T>
where
    T: Send + Reloc,
{
    fn cmp(&self, other: &Self) -> Ordering {
        self.id.cmp(&other.id)
    }
}

pub struct IceoryxSampleMut<'a, T>
where
    T: Reloc,
{
    data: T,
    _lifetime: PhantomData<&'a T>,
}

impl<'a, T> SampleMutConcept<T> for IceoryxSampleMut<'a, T>
where
    T: Reloc + Send,
{
    type Sample = IceoryxSample<'a, T>;

    fn into_sample(self) -> Self::Sample {
        todo!()
    }

    fn send(self) -> com_api_concept::Result<()> {
        todo!()
    }
}

impl<'a, T> Deref for IceoryxSampleMut<'a, T>
where
    T: Reloc,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<'a, T> DerefMut for IceoryxSampleMut<'a, T>
where
    T: Reloc,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}

pub struct IceoryxSampleMaybeUninit<'a, T>
where
    T: Reloc + Send,
{
    data: MaybeUninit<T>,
    _lifetime: PhantomData<&'a T>,
}

impl<'a, T> SampleMaybeUninitConcept<T> for IceoryxSampleMaybeUninit<'a, T>
where
    T: Reloc + Send,
{
    type SampleMut = IceoryxSampleMut<'a, T>;

    fn write(self, val: T) -> IceoryxSampleMut<'a, T> {
        IceoryxSampleMut {
            data: val,
            _lifetime: PhantomData,
        }
    }
}

pub struct IceoryxSubscribable<T> {
    _data: PhantomData<T>,
}

impl<T> Default for IceoryxSubscribable<T> {
    fn default() -> Self {
        Self { _data: PhantomData }
    }
}

impl<T: Reloc + Send> SubscriberConcept<T> for IceoryxSubscribable<T> {
    type Subscription = IceoryxSubscriber<T>;

    fn subscribe(self, _max_num_samples: usize) -> com_api_concept::Result<Self::Subscription> {
        Ok(IceoryxSubscriber::new())
    }
}

#[derive(Default)]
pub struct IceoryxSubscriber<T>
where
    T: Reloc + Send,
{
    data: VecDeque<T>,
}

impl<T> IceoryxSubscriber<T>
where
    T: Reloc + Send,
{
    pub fn new() -> Self {
        Self {
            data: Default::default(),
        }
    }

    pub fn add_data(&mut self, data: T) {
        self.data.push_front(data);
    }
}

impl<T> SubscriptionConcept<T> for IceoryxSubscriber<T>
where
    T: Reloc + Send,
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
        todo!()
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

pub struct IceoryxPublisher<T> {
    _data: PhantomData<T>,
}

impl<T> Default for IceoryxPublisher<T>
where
    T: Reloc + Send,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<T> IceoryxPublisher<T>
where
    T: Reloc + Send,
{
    pub fn new() -> Self {
        Self { _data: PhantomData }
    }

    pub fn allocate<'a>(&'a self) -> com_api_concept::Result<IceoryxSampleMaybeUninit<'a, T>> {
        Ok(IceoryxSampleMaybeUninit {
            data: MaybeUninit::uninit(),
            _lifetime: PhantomData,
        })
    }
}

pub struct IceoryxConsumerDiscovery<I> {
    _interface: PhantomData<I>,
}

impl<I> IceoryxConsumerDiscovery<I> {
    fn new(_runtime: &IceoryxAdapter, _instance_specifier: InstanceSpecifier) -> Self {
        Self {
            _interface: PhantomData,
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
        Ok(Vec::new())
    }
}

pub struct IceoryxProducerBuilder<I: InterfaceConcept> {
    instance_specifier: InstanceSpecifier,
    _interface: PhantomData<I>,
}

impl<I: InterfaceConcept> IceoryxProducerBuilder<I> {
    fn new(_runtime: &IceoryxAdapter, instance_specifier: InstanceSpecifier) -> Self {
        Self {
            instance_specifier,
            _interface: PhantomData,
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
    instance_specifier: InstanceSpecifier,
    _interface: PhantomData<I>,
}

impl<I: InterfaceConcept> ConsumerDescriptorConcept<IceoryxAdapter> for IceoryxConsumerBuilder<I> {
    fn get_instance_id(&self) -> usize {
        todo!()
    }
}

pub struct IceoryxAdapterBuilder {}

impl BuilderConcept<IceoryxAdapter> for IceoryxAdapterBuilder {
    fn build(self) -> com_api_concept::Result<IceoryxAdapter> {
        Ok(IceoryxAdapter {})
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
