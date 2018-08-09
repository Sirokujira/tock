use core::cell::Cell;
use core::ops::{Index, IndexMut};

use kernel::common::StaticRef;
use kernel::common::registers::{self, ReadOnly, ReadWrite, Field, FieldValue};
use kernel::hil;
use kernel::common::cells::OptionalCell;

#[repr(C)]
struct GpioRegisters {
	/// Pin value.
	value: ReadOnly<u32, pins::Register>,
	/// Pin Input Enable Register
	input_en: ReadWrite<u32, pins::Register>,
	/// Pin Output Enable Register
	output_en: ReadWrite<u32, pins::Register>,
	/// Output Port Value Register
	port: ReadWrite<u32, pins::Register>,
	/// Internal Pull-Up Enable Register
	pullup: ReadWrite<u32, pins::Register>,
	/// Drive Strength Register
	drive: ReadWrite<u32, pins::Register>,
	/// Rise Interrupt Enable Register
	rise_ie: ReadWrite<u32, pins::Register>,
	/// Rise Interrupt Pending Register
	rise_ip: ReadWrite<u32, pins::Register>,
	/// Fall Interrupt Enable Register
	fall_ie: ReadWrite<u32, pins::Register>,
	/// Fall Interrupt Pending Register
	fall_ip: ReadWrite<u32, pins::Register>,
	/// High Interrupt Enable Register
	high_ie: ReadWrite<u32, pins::Register>,
	/// High Interrupt Pending Register
	high_ip: ReadWrite<u32, pins::Register>,
	/// Low Interrupt Enable Register
	low_ie: ReadWrite<u32, pins::Register>,
	/// Low Interrupt Pending Register
	low_ip: ReadWrite<u32, pins::Register>,
	/// HW I/O Function Enable Register
	iof_en: ReadWrite<u32, pins::Register>,
	/// HW I/O Function Select Register
	iof_sel: ReadWrite<u32, pins::Register>,
	/// Output XOR (invert) Register
	out_xor: ReadWrite<u32, pins::Register>,
}

register_bitfields![u32,
	pins [
	    pin0 0,
	    pin1 1,
	    pin2 2,
	    pin3 3,
	    pin4 4,
	    pin5 5,
	    pin6 6,
	    pin7 7,
	    pin8 8,
	    pin9 9,
	    pin10 10,
	    pin11 11,
	    pin12 12,
	    pin13 13,
	    pin14 14,
	    pin15 15,
	    pin16 16,
	    pin17 17,
	    pin18 18,
	    pin19 19,
	    pin20 20,
	    pin21 21,
	    pin22 22,
	    pin23 23,
	    pin24 24,
	    pin25 25,
	    pin26 26,
	    pin27 27,
	    pin28 28,
	    pin29 29,
	    pin30 30,
	    pin31 31
	]
];

const GPIO0_BASE: StaticRef<GpioRegisters> =
    unsafe { StaticRef::new(0x1001_2000 as *const GpioRegisters) };

pub struct Port {
    pins: [GpioPin; 32],
}

impl Index<usize> for Port {
    type Output = GpioPin;

    fn index(&self, index: usize) -> &GpioPin {
        &self.pins[index]
    }
}

impl IndexMut<usize> for Port {
    fn index_mut(&mut self, index: usize) -> &mut GpioPin {
        &mut self.pins[index]
    }
}

pub static mut PORT: Port = Port {
    pins: [
        GpioPin::new(pins::pin0, pins::pin0::SET, pins::pin0::CLEAR),
        GpioPin::new(pins::pin1, pins::pin1::SET, pins::pin1::CLEAR),
        GpioPin::new(pins::pin2, pins::pin2::SET, pins::pin2::CLEAR),
        GpioPin::new(pins::pin3, pins::pin3::SET, pins::pin3::CLEAR),
        GpioPin::new(pins::pin4, pins::pin4::SET, pins::pin4::CLEAR),
        GpioPin::new(pins::pin5, pins::pin5::SET, pins::pin5::CLEAR),
        GpioPin::new(pins::pin6, pins::pin6::SET, pins::pin6::CLEAR),
        GpioPin::new(pins::pin7, pins::pin7::SET, pins::pin7::CLEAR),
        GpioPin::new(pins::pin8, pins::pin8::SET, pins::pin8::CLEAR),
        GpioPin::new(pins::pin9, pins::pin9::SET, pins::pin9::CLEAR),
        GpioPin::new(pins::pin10, pins::pin10::SET, pins::pin10::CLEAR),
        GpioPin::new(pins::pin11, pins::pin11::SET, pins::pin11::CLEAR),
        GpioPin::new(pins::pin12, pins::pin12::SET, pins::pin12::CLEAR),
        GpioPin::new(pins::pin13, pins::pin13::SET, pins::pin13::CLEAR),
        GpioPin::new(pins::pin14, pins::pin14::SET, pins::pin14::CLEAR),
        GpioPin::new(pins::pin15, pins::pin15::SET, pins::pin15::CLEAR),
        GpioPin::new(pins::pin16, pins::pin16::SET, pins::pin16::CLEAR),
        GpioPin::new(pins::pin17, pins::pin17::SET, pins::pin17::CLEAR),
        GpioPin::new(pins::pin18, pins::pin18::SET, pins::pin18::CLEAR),
        GpioPin::new(pins::pin19, pins::pin19::SET, pins::pin19::CLEAR),
        GpioPin::new(pins::pin20, pins::pin20::SET, pins::pin20::CLEAR),
        GpioPin::new(pins::pin21, pins::pin21::SET, pins::pin21::CLEAR),
        GpioPin::new(pins::pin22, pins::pin22::SET, pins::pin22::CLEAR),
        GpioPin::new(pins::pin23, pins::pin23::SET, pins::pin23::CLEAR),
        GpioPin::new(pins::pin24, pins::pin24::SET, pins::pin24::CLEAR),
        GpioPin::new(pins::pin25, pins::pin25::SET, pins::pin25::CLEAR),
        GpioPin::new(pins::pin26, pins::pin26::SET, pins::pin26::CLEAR),
        GpioPin::new(pins::pin27, pins::pin27::SET, pins::pin27::CLEAR),
        GpioPin::new(pins::pin28, pins::pin28::SET, pins::pin28::CLEAR),
        GpioPin::new(pins::pin29, pins::pin29::SET, pins::pin29::CLEAR),
        GpioPin::new(pins::pin30, pins::pin30::SET, pins::pin30::CLEAR),
        GpioPin::new(pins::pin31, pins::pin31::SET, pins::pin31::CLEAR),
    ],
};

pub struct GpioPin {
    registers: StaticRef<GpioRegisters>,
    pin: Field<u32, pins::Register>,
    set: FieldValue<u32, pins::Register>,
    clear: FieldValue<u32, pins::Register>,
    client_data: Cell<usize>,
    client: OptionalCell<&'static hil::gpio::Client>,
}

impl GpioPin {
    const fn new(pin: Field<u32, pins::Register>, set: FieldValue<u32, pins::Register>, clear: FieldValue<u32, pins::Register>) -> GpioPin {
        GpioPin {
        	registers: GPIO0_BASE,
            pin: pin,
            set: set,
            clear: clear,
            client_data: Cell::new(0),
            client: OptionalCell::empty(),
        }
    }

    pub fn set_client<C: hil::gpio::Client>(&self, client: &'static C) {
        self.client.set(client);
    }
}

impl hil::gpio::PinCtl for GpioPin {
    fn set_input_mode(&self, mode: hil::gpio::InputMode) {
        match mode {
            hil::gpio::InputMode::PullUp => {
            	self.registers.pullup.modify(self.set);
            	// self.registers.input_en.write(self.pin::SET);
            	// self.registers.iof_en.write(self.pin::CLEAR);
                // self.disable_pull_down();
                // self.enable_pull_up();
            }
            hil::gpio::InputMode::PullDown => {
            	self.registers.pullup.modify(self.clear);
                // self.disable_pull_up();
                // self.enable_pull_down();

            }
            hil::gpio::InputMode::PullNone => {
                self.registers.pullup.modify(self.clear);
            }
        }
    }
}

impl hil::gpio::Pin for GpioPin {
    fn disable(&self) {
        // nop maybe?
    }

    fn make_output(&self) {
    	self.registers.drive.modify(self.clear);
    	self.registers.out_xor.modify(self.clear);
    	self.registers.output_en.modify(self.set);
    	self.registers.iof_en.modify(self.clear);


    	// self.registers.drive.set(0);
    	// self.registers.out_xor.set(0);
    	// self.registers.output_en.set(1<<19);
    	// self.registers.iof_en.set(0);

    	// drive.drive().modify(|_, w| w.$pxi().bit(false));
     //                    out_xor.out_xor().modify(|_, w| w.$pxi().bit(false));
     //                    output_en.output_en().modify(|_, w| w.$pxi().bit(true));
     //                    iof_en.iof_en().modify(|_, w| w.$pxi().bit(false));



     //    self.enable();
     //    GPIOPin::enable_output(self);
     //    self.disable_schmidtt_trigger();
    }

    fn make_input(&self) {
    	self.registers.pullup.modify(self.clear);
    	self.registers.input_en.modify(self.set);
    	self.registers.iof_en.modify(self.clear);


    	// pullup.pullup().modify(|_, w| w.$pxi().bit(false));
    	// input_en.input_en().modify(|_, w| w.$pxi().bit(true));
    	// iof_en.iof_en().modify(|_, w| w.$pxi().bit(false));


     //    self.enable();
     //    GPIOPin::disable_output(self);
     //    self.enable_schmidtt_trigger();
    }

    fn read(&self) -> bool {
    	self.registers.value.is_set(self.pin)
        // GPIOPin::read(self)
    }

    fn toggle(&self) {
    	let current_outputs = self.registers.port.extract();
    	if current_outputs.is_set(self.pin) {
    		self.registers.port.modify_no_read(current_outputs, self.clear);
    	} else {
    		self.registers.port.modify_no_read(current_outputs, self.set);
    	}
        // GPIOPin::toggle(self);
    }

    fn set(&self) {
    	self.registers.port.modify(self.set);


    	// self.registers.port.set(1<<19);

        // GPIOPin::set(self);
    }

    fn clear(&self) {
    	self.registers.port.modify(self.clear);


    	// self.registers.port.set(0);


        // GPIOPin::clear(self);
    }

    fn enable_interrupt(&self, client_data: usize, mode: hil::gpio::InterruptMode) {
        // let mode_bits = match mode {
        //     hil::gpio::InterruptMode::EitherEdge => 0b00,
        //     hil::gpio::InterruptMode::RisingEdge => 0b01,
        //     hil::gpio::InterruptMode::FallingEdge => 0b10,
        // };
        // self.client_data.set(client_data);
        // GPIOPin::set_interrupt_mode(self, mode_bits);
        // GPIOPin::enable_interrupt(self);
    }

    fn disable_interrupt(&self) {
        // GPIOPin::disable_interrupt(self);
    }
}


