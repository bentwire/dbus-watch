use dbus::blocking::Connection;
use std::time::Duration;
use dbus::{Message, Path};
use dbus::message::MatchRule;
use dbus::arg::{ReadAll, RefArg, Variant};
use std::borrow::Borrow;

mod networkmanager;
mod udisks2;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Hello, world!");

    // Connect to the system bus to watch for NetworkManager events.

    let mut conn = Connection::new_system()?;
    //let mut conn = Connection::new_session()?;

    let proxy = conn.with_proxy("org.freedesktop.DBus.Properties", "/", Duration::from_millis(5000));

    let _id = proxy.match_signal(|sig: networkmanager::OrgFreedesktopDBusPropertiesPropertiesChanged, _conn: &Connection, _msg: &Message| {
        println!("MATCH: {:?}", sig);
        true
    });

    //conn.add_match_no_cb("interface='org.freedesktop.DBus.Properties'")?;

    //let mut matchrule = MatchRule::new_signal("org.freedesktop.DBus.Properties", "PropertiesChanged");
    let mut matchrule = MatchRule::new_signal("org.freedesktop.DBus.ObjectManager", "InterfacesAdded");
    //matchrule.path_is_namespace = true;
    //matchrule.path = Some(Path::new("/org/freedesktop/UDisks2")?);

    //conn.add_match(matchrule, |arg: udisks2::OrgFreedesktopDBusPropertiesPropertiesChanged, z, msg: &Message| {
    conn.add_match(matchrule, |arg: udisks2::OrgFreedesktopDBusObjectManagerInterfacesAdded, z, msg: &Message| {
        println!("I MATCH!");
        if let Some(blockdata) = arg.interfaces_and_properties.get("org.freedesktop.UDisks2.Block") {
            println!("BD: {:?}", blockdata);
            if let Some(size) = blockdata.get("Size") {
                println!("SIZE: {:?}", size.as_u64().unwrap())
            }
            if let Some(device) = blockdata.get("PreferredDevice") {
                let device: & Variant<Box<dyn RefArg + 'static>> = device;
                let device = &device.0;
                let device = dbus::arg::cast::<Vec<u8>>(device);
                println!("DEVICE: {:?}", std::str::from_utf8(device.unwrap()));
            }
        }
        //println!("ARG: {:?}", arg.interfaces_and_properties);
        //println!("CON: {:?}", z.);
        //println!("MSG: {:?}", msg);

        true
    });

    loop {
        let _ret = conn.process(Duration::from_millis(1000))?;//process(Duration::from_millis(5000))?;

        //println!("RET: {:?}", ret);
    }

    // // Second, create a wrapper struct around the connection that makes it easy
    // // to send method calls to a specific destination and path.
    // let proxy = conn.with_proxy("org.freedesktop.DBus", "/", Duration::from_millis(5000));
    //
    // // Now make the method call. The ListNames method call takes zero input parameters and
    // // one output parameter which is an array of strings.
    // // Therefore the input is a zero tuple "()", and the output is a single tuple "(names,)".
    // let (names,): (Vec<String>,) = proxy.method_call("org.freedesktop.DBus", "ListNames", ())?;
    //
    // // Let's print all the names to stdout.
    // for name in names { println!("{}", name); }
    //
    // loop {
    //     let ret = conn.process(Duration::from_millis(1000))?;
    //
    //     println!("RET: {:?}", ret);
    // }

    // use dbus::{ffidisp::Connection, Message, MessageType};
    //
    // fn focus_msg(msg: &Message) -> Option<&str> {
    //     if msg.msg_type() != MessageType::Signal { return None };
    //     if &*msg.interface().unwrap() != "com.canonical.Unity.WindowStack" { return None };
    //     if &*msg.member().unwrap() != "FocusedWindowChanged" { return None };
    //     let (_, app) = msg.get2::<u32, &str>();
    //     app
    // }
    //
    // let c = Connection::new_system().unwrap();
    // c.add_match("interface='org.freedesktop.NetworkManager.AccessPoint'").unwrap();
    //
    // loop {
    //     if let Some(msg) = c.incoming(1000).next() {
    //         println!("MSG: {:?}", msg);
    //         if let Some(app) = focus_msg(&msg) {
    //             println!("{} has now focus.", app);
    //         }
    //     }
    // }
    Ok(())
}
