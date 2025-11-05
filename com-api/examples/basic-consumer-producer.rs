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

use com_api::*;

fn main() {
    lola_run();
    mock_run();
}

fn lola_run() {
    use com_api_gen_lola::*;
    use com_api_runtime_lola::{LolaRuntimeImpl, RuntimeBuilderImpl as LolaRuntimeBuilderImpl};

    let runtime_builder = LolaRuntimeBuilderImpl::new();
    let runtime = Builder::<LolaRuntimeImpl>::build(runtime_builder).unwrap();
    let producer_builder = runtime.producer_builder::<VehicleInterface>(InstanceSpecifier {
        specifier: "My/Funk/ServiceName".to_string(),
    });
    let producer = producer_builder.get_builder().build().unwrap();
    let offered_producer = producer.offer().unwrap();

    // Business logic
    let uninit_sample = offered_producer.left_tire.allocate().unwrap();
    let sample = uninit_sample.write(Tire {});
    sample.send().unwrap();

    // Create service discovery
    let consumer_discovery = runtime.find_service::<VehicleInterface>(InstanceSpecifier {
        specifier: "My/Funk/ServiceName".to_string(),
    });
    let available_service_instances = consumer_discovery.get_available_instances().unwrap();

    // Create consumer from first discovered service
    let consumer_builder = available_service_instances
        .into_iter()
        .find(|desc| desc.get_instance_id() == 42)
        .unwrap();
    let consumer = consumer_builder.get_builder().build().unwrap();

    // Subscribe to one event
    let subscribed = consumer.left_tire.subscribe(3).unwrap();

    // Create sample buffer to be used during receive
    let mut sample_buf = SampleContainer::new();
    for _ in 0..10 {
        let uninit_sample = offered_producer.left_tire.allocate().unwrap();
        let sample = uninit_sample.write(Tire {});
        sample.send().unwrap();
        match subscribed.try_receive(&mut sample_buf, 1) {
            Ok(0) => panic!("No sample received"),
            Ok(x) => {
                let sample = sample_buf.pop_front().unwrap();
                println!("{} sample received: sample[0] = {:?}", x, *sample)
            }
            Err(e) => panic!("{:?}", e),
        }
    }
}

fn mock_run() {
    use com_api_gen_mock::*;
    use com_api_runtime_mock::{MockRuntimeImpl, RuntimeBuilderImpl as MockRuntimeBuilderImpl};

    let runtime_builder = MockRuntimeBuilderImpl::new();
    let runtime = Builder::<MockRuntimeImpl>::build(runtime_builder).unwrap();
    let producer_builder = runtime.producer_builder::<VehicleInterface>(InstanceSpecifier {
        specifier: "My/Funk/ServiceName".to_string(),
    });
    let producer = producer_builder.get_builder().build().unwrap();
    let offered_producer = producer.offer().unwrap();

    // Business logic
    let uninit_sample = offered_producer.left_tire.allocate().unwrap();
    let sample = uninit_sample.write(Tire {});
    sample.send().unwrap();

    // Create service discovery
    let consumer_discovery = runtime.find_service::<VehicleInterface>(InstanceSpecifier {
        specifier: "My/Funk/ServiceName".to_string(),
    });
    let available_service_instances = consumer_discovery.get_available_instances().unwrap();

    // Create consumer from first discovered service
    let consumer_builder = available_service_instances
        .into_iter()
        .find(|desc| desc.get_instance_id() == 42)
        .unwrap();
    let consumer = consumer_builder.get_builder().build().unwrap();

    // Subscribe to one event
    let subscribed = consumer.left_tire.subscribe(3).unwrap();

    // Create sample buffer to be used during receive
    let mut sample_buf = SampleContainer::new();
    for _ in 0..10 {
        let uninit_sample = offered_producer.left_tire.allocate().unwrap();
        let sample = uninit_sample.write(Tire {});
        sample.send().unwrap();
        match subscribed.try_receive(&mut sample_buf, 1) {
            Ok(0) => panic!("No sample received"),
            Ok(x) => {
                let sample = sample_buf.pop_front().unwrap();
                println!("{} sample received: sample[0] = {:?}", x, *sample)
            }
            Err(e) => panic!("{:?}", e),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn single_thread_transceiver<T: Runtime>(runtime: T) {
        let producer_builder = runtime.producer_builder::<VehicleInterface>(InstanceSpecifier {
            specifier: "My/Funk/ServiceName".to_string(),
        });
        let producer = producer_builder.get_builder().build().unwrap();
        let offered_producer = producer.offer().unwrap();

        // Business logic
        let uninit_sample = offered_producer.left_tire.allocate().unwrap();
        let sample = uninit_sample.write(Tire {});
        sample.send().unwrap();

        // Create service discovery
        let consumer_discovery = runtime.find_service::<VehicleInterface>(InstanceSpecifier {
            specifier: "My/Funk/ServiceName".to_string(),
        });
        let available_service_instances = consumer_discovery.get_available_instances().unwrap();

        // Create consumer from first discovered service
        let consumer_builder = available_service_instances
            .into_iter()
            .find(|desc| desc.get_instance_id() == 42)
            .unwrap();
        let consumer = consumer_builder.build().unwrap();

        // Subscribe to one event
        let subscribed = consumer.left_tire.subscribe(3).unwrap();

        // Create sample buffer to be used during receive
        let mut sample_buf = SampleContainer::new();
        for _ in 0..10 {
            let uninit_sample = offered_producer.left_tire.allocate().unwrap();
            let sample = uninit_sample.write(com_api_gen_mock::Tire {});
            sample.send().unwrap();
            match subscribed.try_receive(&mut sample_buf, 1) {
                Ok(0) => panic!("No sample received"),
                Ok(x) => {
                    let sample = sample_buf.pop_front().unwrap();
                    println!("{} sample received: sample[0] = {:?}", x, *sample)
                }
                Err(e) => panic!("{:?}", e),
            }
        }
    }

    #[test]
    fn multi_runtime_test() {
        use com_api_gen_lola::*;
        use com_api_gen_mock::*;
        use com_api_runtime_lola::{LolaRuntimeImpl, RuntimeBuilderImpl as LolaRuntimeBuilderImpl};
        use com_api_runtime_mock::{MockRuntimeImpl, RuntimeBuilderImpl as MockRuntimeBuilderImpl};
        // Factory
        let runtime_builder = MockRuntimeBuilderImpl::new();
        let runtime = Builder::<MockRuntimeImpl>::build(runtime_builder).unwrap();
        single_thread_transceiver(runtime);
        let runtime_builder = LolaRuntimeBuilderImpl::new();
        let runtime = Builder::<LolaRuntimeImpl>::build(runtime_builder).unwrap();
        single_thread_transceiver(runtime);
    }

    #[test]
    fn create_producer() {
        // Factory
        let runtime_builder = MockRuntimeBuilderImpl::new();
        let runtime = runtime_builder.build().unwrap();
        let producer_builder = runtime.producer_builder::<VehicleInterface>(InstanceSpecifier {
            specifier: "My/Funk/ServiceName".to_string(),
        });
        let producer = producer_builder.get_builder().build().unwrap();
        let offered_producer = producer.offer().unwrap();

        // Business logic
        let uninit_sample = offered_producer.left_tire.allocate().unwrap();
        let sample = uninit_sample.write(Tire {});
        sample.send().unwrap();
    }

    #[test]
    fn create_consumer() {
        // Create runtime
        let runtime_builder = MockRuntimeBuilderImpl::new();
        let runtime = runtime_builder.build().unwrap();

        // Create service discovery
        let consumer_discovery = runtime.find_service::<VehicleInterface>(InstanceSpecifier {
            specifier: "My/Funk/ServiceName".to_string(),
        });
        let available_service_instances = consumer_discovery.get_available_instances().unwrap();

        // Create consumer from first discovered service
        let consumer_builder = available_service_instances
            .into_iter()
            .find(|desc| desc.get_instance_id() == 42)
            .unwrap();
        let consumer = consumer_builder.get_builder().build().unwrap();

        // Subscribe to one event
        let subscribed = consumer.left_tire.subscribe(3).unwrap();

        // Create sample buffer to be used during receive
        let mut sample_buf = SampleContainer::new();
        for _ in 0..10 {
            match subscribed.try_receive(&mut sample_buf, 1) {
                Ok(0) => panic!("No sample received"),
                Ok(x) => {
                    let sample = sample_buf.pop_front().unwrap();
                    println!("{} samples received: sample[0] = {:?}", x, *sample)
                }
                Err(e) => panic!("{:?}", e),
            }
        }
    }

    async fn async_data_processor_fn(subscribed: impl Subscription<Tire>) {
        let mut buffer = SampleContainer::new();
        for _ in 0..10 {
            match subscribed.receive(&mut buffer, 1, 1).await {
                Ok(0) => panic!("No sample received"),
                Ok(num_samples) => {
                    println!(
                        "{} samples received: sample[0] = {:?}",
                        num_samples,
                        *buffer.front().unwrap()
                    );
                }
                Err(e) => panic!("{:?}", e),
            }
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn schedule_subscription_on_mt_scheduler() {
        let runtime_builder = MockRuntimeBuilderImpl::new();
        let runtime = runtime_builder.build().unwrap();

        let consumer_discovery = runtime.find_service::<VehicleInterface>(InstanceSpecifier {
            specifier: "My/Funk/ServiceName".to_string(),
        });
        let available_service_instances = consumer_discovery.get_available_instances().unwrap();

        // Create consumer from first discovered service
        let consumer_builder = available_service_instances
            .into_iter()
            .find(|desc| desc.get_instance_id() == 42)
            .unwrap();
        let consumer = consumer_builder.get_builder().build().unwrap();

        // Subscribe to one event
        let subscribed = consumer.left_tire.subscribe(3).unwrap();

        tokio::spawn(async_data_processor_fn(subscribed))
            .await
            .expect("Error returned from task");
    }
}
