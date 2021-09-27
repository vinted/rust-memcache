extern crate memcache;
extern crate rand;

use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use std::collections::HashMap;
use std::iter;
use std::thread;
use std::thread::JoinHandle;
use std::time;

mod helpers;

fn gen_random_key() -> String {
    let bs = iter::repeat(())
        .map(|()| thread_rng().sample(Alphanumeric))
        .take(10)
        .collect::<Vec<u8>>();
    return String::from_utf8(bs).unwrap();
}

#[test]
fn udp_test() {
    let client = helpers::connect("memcache+udp://localhost:22345").unwrap();

    client.version().unwrap();

    client.set("foo", "bar", 0).unwrap();
    client.flush().unwrap();
    let value: Option<String> = client.get("foo").unwrap();
    assert_eq!(value, None);

    client.set("foo", "bar", 0).unwrap();
    client.flush_with_delay(3).unwrap();
    let value: Option<String> = client.get("foo").unwrap();
    assert_eq!(value, Some(String::from("bar")));
    thread::sleep(time::Duration::from_secs(4));
    let value: Option<String> = client.get("foo").unwrap();
    assert_eq!(value, None);

    client.set("foo", "bar", 0).unwrap();
    let value = client.add("foo", "baz", 0);
    assert_eq!(value.is_err(), true);

    client.delete("foo").unwrap();
    let value: Option<String> = client.get("foo").unwrap();
    assert_eq!(value, None);

    client.add("foo", "bar", 0).unwrap();
    let value: Option<String> = client.get("foo").unwrap();
    assert_eq!(value, Some(String::from("bar")));

    client.replace("foo", "baz", 0).unwrap();
    let value: Option<String> = client.get("foo").unwrap();
    assert_eq!(value, Some(String::from("baz")));

    assert_eq!(client.touch("foooo", 123).unwrap(), false);
    assert_eq!(client.touch("fooo", 12345).unwrap(), true);

    let mut keys: Vec<String> = Vec::new();
    for _ in 0..1000 {
        let key = gen_random_key();
        keys.push(key.clone());
        client.set(key.as_str(), "xxx", 0).unwrap();
    }

    for key in keys {
        let value: String = client.get(key.as_str()).unwrap().unwrap();

        assert_eq!(value, "xxx");
    }

    // test with multiple udp connections
    let mut handles: Vec<Option<JoinHandle<_>>> = Vec::new();
    for i in 0..10 {
        handles.push(Some(thread::spawn(move || {
            let key = format!("key{}", i);
            let value = format!("value{}", i);
            let client = helpers::connect("memcache://localhost:22345?udp=true").unwrap();
            for j in 0..50 {
                let value = format!("{}{}", value, j);
                client.set(key.as_str(), &value, 0).unwrap();
                let result: Option<String> = client.get(key.as_str()).unwrap();
                assert_eq!(result.as_ref(), Some(&value));

                let result = client.add(key.as_str(), &value, 0);
                assert_eq!(result.is_err(), true);

                client.delete(key.as_str()).unwrap();
                let result: Option<String> = client.get(key.as_str()).unwrap();
                assert_eq!(result, None);

                client.add(key.as_str(), &value, 0).unwrap();
                let result: Option<String> = client.get(key.as_str()).unwrap();
                assert_eq!(result.as_ref(), Some(&value));

                client.replace(key.as_str(), &value, 0).unwrap();
                let result: Option<String> = client.get(key.as_str()).unwrap();
                assert_eq!(result.as_ref(), Some(&value));
            }
        })));
    }

    for i in 0..10 {
        handles[i].take().unwrap().join().unwrap();
    }
}

// // TODO: enable
// #[test]
// fn test_cas() {
//     let clients = vec![
//         helpers::connect("memcache://localhost:12345").unwrap(),
//         helpers::connect("memcache://localhost:12345?protocol=ascii").unwrap(),
//     ];
//     for client in clients {
//         client.flush().unwrap();
//
//         client.set("ascii_foo", "bar", 0).unwrap();
//         let value: Option<String> = client.get("ascii_foo").unwrap();
//         assert_eq!(value, Some("bar".into()));
//
//         client.set("ascii_baz", "qux", 0).unwrap();
//
//         assert!(ascii_foo_value.2.is_some());
//         assert!(ascii_baz_value.2.is_some());
//         assert_eq!(
//             true,
//             client.cas("ascii_foo", "bar2", 0, ascii_foo_value.2.unwrap()).unwrap()
//         );
//         assert_eq!(
//             false,
//             client.cas("ascii_foo", "bar3", 0, ascii_foo_value.2.unwrap()).unwrap()
//         );
//
//         assert_eq!(
//             false,
//             client
//                 .cas("not_exists_key", "bar", 0, ascii_foo_value.2.unwrap())
//                 .unwrap()
//         );
//         client.flush().unwrap();
//     }
// }
