/*
 * Copyright (C) 2019 Richard Hughes <richard@hughsie.com>
 */

#![no_main]
#![no_std]
#![allow(stable_features)]
#![allow(unused_imports)]
#![allow(dead_code)]
#![allow(unreachable_code)]
#![allow(unused_variables)]

use log::info;
use log::warn;
use uefi::prelude::*;
use uefi::proto::device_path::text::{AllowShortcuts, DevicePathToText, DisplayOnly};
use uefi::proto::loaded_image::LoadedImage;
use uefi::table::boot::SearchType;
use uefi::table::runtime::ResetType;
use uefi::table::runtime::Time;
use uefi::table::runtime::VariableAttributes;
use uefi::table::runtime::VariableVendor;
use uefi::{guid, Guid};
use uefi::{Identify, Result};


extern crate alloc;
use alloc::{vec, vec::Vec};


use uefi::CStr16;

const VERSION: &str = env!("CARGO_PKG_VERSION");

const FWUPDATE_GUID: Guid = guid!("0abba7dc-e516-4167-bbf5-4d9d1c739416");
const UX_CAPSULE_GUID: Guid = guid!("3b8c8162-188c-46a4-aec9-be43f1d65697");

const FWUP_NUM_CAPSULE_UPDATES_MAX: u32 = 128;

const FWUPDATE_ATTEMPT_UPDATE: u32 = 0x1;
const FWUPDATE_ATTEMPTED: u32 = 0x2;

static EXAMPLE_KEY: &'static [u8] = include_bytes!("../fwupd-6e58e73d-8061-44e4-8949-33b7f0d5c726-0-0abba7dc-e516-4167-bbf5-4d9d1c739416");

#[repr(C, packed)]
struct UxCapsuleHeader {
    version: u8,
    checksum: u8,
    image_type: u8,
    reserved: u8,
    mode: u32,
    x_offset: u32,
    y_offset: u32,
}

#[repr(C, packed)]
#[derive(Clone, Debug)]
struct FwupUpdateInfo {
    update_info_version: u32,

    // stuff we need to apply an update
    guid: Guid,
    capsule_flags: u32,
    hw_inst: u64,
    time_attempted: Time,

    // our metadata
    status: u32,

    // variadic device path
    //FIXME dp: DevicePath,
}

struct FwupUpdateTable<'a> {
    name: &'a CStr16,
    attrs: u32,
    size: usize,
    info: FwupUpdateInfo,
}

//use alloc::{vec, vec::Vec};


//fn get_variable_alloc(rt: &RuntimeServices, name: &CStr16, vendor: &VariableVendor) -> u8 {

    //let sz = rt.get_variable_size(name, vendor)?;
//Result<(Vec<u8>, VariableAttributes)>
  //  let mut buf = vec![0; sz];

//    rt.get_variable(name),
  //          &VariableVendor(FWUPDATE_GUID),
    //        &mut buf)?
    //buf
//}

//#[cfg(feature = "alloc")]
fn fwup_populate_update_info<'a>(rt: &RuntimeServices, name: &'a CStr16) -> FwupUpdateTable<'a> {
//    let info = FwupUpdateInfo{};

    let sz = rt.get_variable_size(name, &VariableVendor(FWUPDATE_GUID)).unwrap();

    //let mut buf = [0; sz];
    //let buf = Vec<u8>;
    let mut buf = vec![0; sz];

    let (buf, attrs) = rt.get_variable(name, &VariableVendor(FWUPDATE_GUID), &mut buf).unwrap();

    let (head, body, _tail) = unsafe { buf.align_to::<FwupUpdateInfo>() };
    assert!(head.is_empty(), "Data was not aligned");
    let info = &body[0];

    //let (data, _) = rt.get_variable(name),
      //      &VariableVendor(FWUPDATE_GUID),
        //    &mut buf).unwrap();

    panic!("FwupUpdateInfo: {:?}", info);

    let update = FwupUpdateTable {
        name: name,
        attrs: attrs.bits(),
        size: 0,
        info: info.clone(),
    };

    update
}

#[entry]
fn main(_image_handle: Handle, mut system_table: SystemTable<Boot>) -> Status {
    uefi_services::init(&mut system_table).unwrap();
    let bs = system_table.boot_services();
    let rt = system_table.runtime_services();

    rt.delete_variable(
        cstr16!("FWUPDATE_DEBUG_LOG"),
        &VariableVendor(FWUPDATE_GUID),
    )
    .unwrap_or_else(|error| {
        if error.status() == uefi::Status::NOT_FOUND {
            info!("no FWUPDATE_DEBUG_LOG to delete!");
        } else {
            panic!("failed to delete FWUPDATE_DEBUG_LOG: {:?}", error);
        }
    });

    // console should be verbose
    let mut buf = [0u8; 1];
    let (data, _) = rt
        .get_variable(
            cstr16!("FWUPDATE_VERBOSE"),
            &VariableVendor(FWUPDATE_GUID),
            &mut buf,
        )
        .unwrap_or_else(|error| {
            if error.status() == uefi::Status::NOT_FOUND {
                ((b"\0"), VariableAttributes::empty())
            } else {
                panic!("failed to get FWUPDATE_VERBOSE: {:?}", error);
            }
        });
    let mut is_debugging = data[0] > 0;
    if true {
        is_debugging = true; // FIXME
    }
    info!("is_debugging: {:?}", is_debugging);

    info!("fwupd-efirs {}", VERSION);

    // FIXME just set this here so we can debug as if a real system
    rt.set_variable(
        cstr16!("fwupd-6e58e73d-8061-44e4-8949-33b7f0d5c726-0"),
        &VariableVendor(FWUPDATE_GUID),
        VariableAttributes::BOOTSERVICE_ACCESS | VariableAttributes::NON_VOLATILE,
        EXAMPLE_KEY,
    ).unwrap();


    // FIXME stuff we'll need in the future
    let time = rt.get_time().unwrap();
    info!("TIME {}", time);

    let variable_keys = rt.variable_keys().expect("failed to get variable keys");
    let mut n_updates = 0;
    for key in variable_keys {

        let key_name = key.name().unwrap();

        // not one of our state variables
        if key.vendor != VariableVendor(FWUPDATE_GUID) {
            continue;
        }

        // ignore debugging settings
        if [cstr16!("FWUPDATE_VERBOSE"), cstr16!("FWUPDATE_DEBUG_LOG")]
            .contains(&key_name)
        {
            continue;
        }

        if n_updates > FWUP_NUM_CAPSULE_UPDATES_MAX {
            warn!("ignoring update: {:?}", key_name);
            continue;
        }

        //
        //match key.name().unwrap() {
        //cstr16!("FWUPDATE_VERBOSE") => {
        //continue;
        //}
        //}
        info!("Found update {}", key_name);

        let update = fwup_populate_update_info(rt, key_name);

        n_updates += 1;
    }

    if n_updates == 0 {
        warn!("No updates to process, exiting in 10 seconds");
        bs.stall(10_000_000);
        return Status::INVALID_PARAMETER;
    }

    // FIXME: step 1: find and validate update state variables
    // FIXME: step 2: Build our data structure and add the capsules to it

    info!("n_updates: {}", n_updates);


    // FIXME: step 3: update the state variables
    // FIXME: step 4: apply the capsules

    print_image_path(bs).unwrap();

    // step 5: if #4 didn't reboot us, do it manually
    info!("Reset System");
    bs.stall(5_000_000);
    if is_debugging {
        bs.stall(30_000_000);
    }
    let reset_type = ResetType::Warm;
    rt.reset(reset_type, Status::SUCCESS, None);

    Status::SUCCESS
}

fn print_image_path(boot_services: &BootServices) -> Result {
    let loaded_image =
        boot_services.open_protocol_exclusive::<LoadedImage>(boot_services.image_handle())?;

    let device_path_to_text_handle = *boot_services
        .locate_handle_buffer(SearchType::ByProtocol(&DevicePathToText::GUID))?
        .first()
        .expect("DevicePathToText is missing");

    let device_path_to_text =
        boot_services.open_protocol_exclusive::<DevicePathToText>(device_path_to_text_handle)?;

    let image_device_path = loaded_image.file_path().expect("File path is not set");
    let image_device_path_text = device_path_to_text
        .convert_device_path_to_text(
            boot_services,
            image_device_path,
            DisplayOnly(true),
            AllowShortcuts(false),
        )
        .expect("convert_device_path_to_text failed");

    info!("Image path: {}", &*image_device_path_text);
    Ok(())
}
