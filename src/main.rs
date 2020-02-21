use dbus::blocking::Connection;
use std::time::Duration;
use dbus::{Message, Path};
use dbus::message::MatchRule;
use dbus::arg::{ReadAll, RefArg, Variant, ArgAll};
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
    let matchadded = MatchRule::new_signal("org.freedesktop.DBus.ObjectManager", "InterfacesAdded");
    let matchremoved = MatchRule::new_signal("org.freedesktop.DBus.ObjectManager", "InterfacesRemoved");
    let matchpropertieschanged = MatchRule::new_signal("org.freedesktop.DBus.Properties", "PropertiesChanged");

    //matchrule.path_is_namespace = true;
    //matchrule.path = Some(Path::new("/org/freedesktop/UDisks2")?);

    //conn.add_match(matchrule, |arg: udisks2::OrgFreedesktopDBusPropertiesPropertiesChanged, z, msg: &Message| {

    fn interfaces_added_watcher(arg: udisks2::OrgFreedesktopDBusObjectManagerInterfacesAdded, _conn: &Connection, _msg: &Message) -> bool {
        //println!("I MATCH: {:?}", arg);

        if let Some(blockdata) = arg.interfaces_and_properties.get("org.freedesktop.UDisks2.Block") {
            println!("BD: {:?}", blockdata);
            if let Some(size) = blockdata.get("Size") {
                println!("\tSIZE: {:?}", size.as_u64().unwrap())
            }
            if let Some(device) = blockdata.get("PreferredDevice") {
                let device: &Variant<Box<dyn RefArg + 'static>> = device;
                let device = &device.0;
                let device = dbus::arg::cast::<Vec<u8>>(device);
                println!("\tDEVICE: {:?}", std::str::from_utf8(device.unwrap()).unwrap());
            }
        }

        let loop_data = arg.interfaces_and_properties.get("org.freedesktop.UDisks2.Loop");
        let partition_data = arg.interfaces_and_properties.get("org.freedesktop.UDisks2.Partition");

        if loop_data.is_some() {
            println!("LD: {:?}", loop_data.unwrap());
        };

        if partition_data.is_some() {
            println!("PD: {:?}", partition_data.unwrap());
            let partition_data = partition_data.unwrap();

            let uuid = &partition_data["UUID"];
            let size = &partition_data["Size"].as_u64().unwrap();
            let table = &partition_data["Table"];
            let name = &partition_data["Name"];
            let offset = &partition_data["Offset"].as_u64().unwrap();
            let type_ = &partition_data["Type"].as_str().unwrap();
            let number = &partition_data["Number"].as_u64().unwrap();

            println!("\tUUID:\t\t{:?}", uuid.as_str().unwrap());
            println!("\tNUMBER:\t\t{:?}", number);
            println!("\tSIZE:\t\t{:?}", size);
            println!("\tTABLE:\t\t{:?}", table.as_str().unwrap());
            println!("\tNAME:\t\t{:?}", name.as_str().unwrap());
            println!("\tOFFSET:\t\t{:?}", offset);
            println!("\tTYPE:\t\t{:?}", type_);

        };

        //println!("ARG: {:?}", arg.interfaces_and_properties);
        //println!("CON: {:?}", z.);
        //println!("MSG: {:?}", msg);

        true
    }

    fn interfaces_removed_watcher(arg: udisks2::OrgFreedesktopDBusObjectManagerInterfacesRemoved, _conn: &Connection, _msg: &Message) -> bool {
        println!("I MATCH: {:?} => {:?}", arg, _msg);
        if arg.interfaces.contains(&"org.freedesktop.UDisks2.Partition".to_owned()) {
            //stuff
            println!("PARTITION_REMOVED: {:?} => {:?}", arg.object_path, _msg.path())
        }

        if arg.interfaces.contains(&"org.freedesktop.UDisks2.Block".to_owned()) {
            //stuff
            println!("BLOCK_REMOVED: {:?}", arg.object_path);
        }

        if arg.interfaces.contains(&"org.freedesktop.UDisks2.PartitionTable".to_owned()) {
            //stuff
            println!("PARTITION_TABLE_REMOVED: {:?}", arg.object_path.to_string());
        }

        true
    }

    fn properties_changed_watcher(arg: udisks2::OrgFreedesktopDBusPropertiesPropertiesChanged, conn: &Connection, msg: &Message) -> bool {
        return match arg.interface_name.as_str() {
            "org.freedesktop.UDisks2.Block" => {
                println!("BLOCK: {:?} => {:?}", msg.path(), arg.changed_properties);
                true
            },
            "org.freedesktop.UDisks2.Loop" => {
                if let Some(backing_file) = arg.changed_properties.get("BackingFile") {
                    let backing_file: &Variant<Box<dyn RefArg + 'static>> = backing_file;
                    let backing_file= dbus::arg::cast::<Vec<u8>>(&backing_file.0).unwrap();

                    println!("LOOP: {:?} => {:?}", msg.path(), std::str::from_utf8(backing_file).unwrap());
                }


                true
            },
            "org.freedesktop.UDisks2.Partition" => {
                println!("PARTITION: {:?}", arg.changed_properties);
                true
            },
            "org.freedesktop.UDisks2.PartitionTable" => {
                println!("PTABLE: {:?}", arg.changed_properties);
                true
            },
            "org.freedesktop.UDisks2.Filesystem" => {
                println!("FILESYSTEM: {:?}", arg.changed_properties);
                true
            },
            "org.freedesktop.systemd1.Device" => {
                println!("DEVICE: {:?} => {:?}", msg.path(), arg.changed_properties);
                true
            },
            "org.freedesktop.systemd1.Unit" => {
                println!("UNIT: {:?} => {:?}", msg.path(), arg.changed_properties);
                true
            },
            // _ => {
            //     true
            // }, // Do nothing.
            str => {
                //println!("Unknown interface_name {:?}", str);
                true
            }
        }
    }

    conn.add_match(matchadded, interfaces_added_watcher)?;
    conn.add_match(matchremoved, interfaces_removed_watcher)?;
    conn.add_match(matchpropertieschanged, properties_changed_watcher)?;

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
