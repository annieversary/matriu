use trellis_m4 as bsp;

use bsp::{
    hal::{
        clock::GenericClockController,
        pac::interrupt,
        usb::{usb_device::bus::UsbBusAllocator, UsbBus},
    },
    pac::{MCLK, USB},
    pins::Usb,
};
use cortex_m::peripheral::NVIC;

use usb_device::prelude::*;
use usbd_midi::{
    data::usb_midi::{
        midi_packet_reader::MidiPacketBufferReader, usb_midi_event_packet::UsbMidiEventPacket,
    },
    midi_device::MidiClass,
};
use usbd_midi::{
    data::{usb::constants::USB_CLASS_NONE, usb_midi::cable_number::CableNumber},
    midi_types::{Channel, MidiMessage, Note, Value7},
};

static mut USB_ALLOCATOR: Option<UsbBusAllocator<UsbBus>> = None;
static mut USB_DEV: Option<UsbDevice<UsbBus>> = None;
static mut USB_MIDI: Option<MidiClass<UsbBus>> = None;

// TODO support note off
// make an enum or smth
// or maybe just wait until usbd-midi updates and uses the new stuff
pub fn send_midi(note: Note, vel: u8, on: bool) {
    let msg = UsbMidiEventPacket {
        cable_number: CableNumber::Cable0,
        message: if on {
            MidiMessage::NoteOn(Channel::new(1), note, Value7::new(vel))
        } else {
            MidiMessage::NoteOff(Channel::new(1), note, Value7::new(vel))
        },
    };
    unsafe {
        let _ = USB_MIDI.as_mut().unwrap().send_message(msg);
    }
}

pub fn setup_usb(
    mclk: &mut MCLK,
    usb_per: USB,
    nvic: &mut NVIC,
    usb: Usb,
    clocks: &mut GenericClockController,
) {
    use bsp::hal::pac::gclk::{genctrl::SRC_A, pchctrl::GEN_A};

    clocks.configure_gclk_divider_and_source(GEN_A::GCLK2, 1, SRC_A::DFLL, false);
    let usb_gclk = clocks.get_gclk(GEN_A::GCLK2).unwrap();
    let usb_clock = &clocks.usb(&usb_gclk).unwrap();

    let usb_allocator = unsafe {
        USB_ALLOCATOR = Some(UsbBusAllocator::new(UsbBus::new(
            usb_clock, mclk, usb.dm, usb.dp, usb_per,
        )));
        USB_ALLOCATOR.as_ref().unwrap()
    };

    unsafe {
        // set interrupts
        // NOTE i don't think we need *all* of these
        // but it works so im not playing with it
        nvic.set_priority(interrupt::USB_OTHER, 1);
        nvic.set_priority(interrupt::USB_TRCPT0, 1);
        nvic.set_priority(interrupt::USB_TRCPT1, 1);
        nvic.set_priority(interrupt::USB_SOF_HSOF, 1);
        NVIC::unmask(interrupt::USB_OTHER);
        NVIC::unmask(interrupt::USB_TRCPT0);
        NVIC::unmask(interrupt::USB_TRCPT1);
        NVIC::unmask(interrupt::USB_SOF_HSOF);

        // set up devices
        USB_MIDI = Some(MidiClass::new(usb_allocator, 1, 1).unwrap());
        USB_DEV = Some(
            UsbDeviceBuilder::new(usb_allocator, UsbVidPid(0x239a, 0x802f))
                .product("annie midi")
                .manufacturer("annieversary")
                .device_class(USB_CLASS_NONE)
                .build(),
        );
    }
}

fn poll_usb() {
    unsafe {
        let Some(usb_dev) = USB_DEV.as_mut()  else {  return;};
        let Some(midi) = USB_MIDI.as_mut() else {  return; };

        if !usb_dev.poll(&mut [midi]) {
            return;
        }

        let mut buffer = [0; 64];

        if let Ok(size) = midi.read(&mut buffer) {
            let buffer_reader = MidiPacketBufferReader::new(&buffer, size);
            for _packet in buffer_reader.into_iter().flatten() {
                // TODO do something here
            }
        }
    };
}

#[interrupt]
fn USB_OTHER() {
    poll_usb();
}

#[interrupt]
fn USB_TRCPT0() {
    poll_usb();
}

#[interrupt]
fn USB_TRCPT1() {
    poll_usb();
}

#[interrupt]
fn USB_SOF_HSOF() {
    poll_usb();
}
