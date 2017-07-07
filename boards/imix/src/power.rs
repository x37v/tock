extern crate kernel;
extern crate sam4l;

use kernel::hil::Controller;
use sam4l::gpio::{PA, PB, PC};
use sam4l::gpio::GPIOPin;
use sam4l::gpio::PeripheralFunction;
use sam4l::gpio::PeripheralFunction::{A, B};

type DetachablePin = (&'static GPIOPin, Option<PeripheralFunction>);

trait Detachable {
    fn detach(&self);
    fn restore(&self, function: Option<PeripheralFunction>);
}

impl Detachable for GPIOPin {
    fn detach(&self) {
        self.configure(None);
        self.enable_output();
        self.clear();
    }

    fn restore(&self, function: Option<PeripheralFunction>) {
        self.configure(function);
    }
}

trait PowerGated {
    fn power(&self, setting: bool);
}

struct ImixSubmodule {
    gate_pin: &'static GPIOPin,
    detachable_pins: Option<&'static [DetachablePin]>,
}

impl ImixSubmodule {
    const fn new(detachable_pins: Option<&'static [DetachablePin]>,
                 gate_pin: &'static GPIOPin)
                 -> ImixSubmodule {
        ImixSubmodule {
            gate_pin: gate_pin,
            detachable_pins: detachable_pins,
        }
    }
}

impl PowerGated for ImixSubmodule {
    fn power(&self, setting: bool) {
        self.gate_pin.enable_output();
        match setting {
            true => {
                if self.detachable_pins.is_some() {
                    for it in self.detachable_pins.unwrap().iter() {
                        let &(pin, function) = it;
                        pin.restore(function);
                    }
                }
                self.gate_pin.set();
            },
            false => {
                self.gate_pin.clear();
                if self.detachable_pins.is_some() {
                    for it in self.detachable_pins.unwrap().iter() {
                        let &(pin, _) = it;
                        pin.detach();
                    }
                }
            },
        }
    }
}

pub struct ModulePowerConfig {
    pub rf233: bool,
    pub nrf51422: bool,
    pub sensors: bool,
    pub trng: bool,
}

pub unsafe fn configure_module_power(enabled_modules: ModulePowerConfig) {
    let rf233_detachable_pins = static_init!([DetachablePin; 3],
                                             [(&PA[08], None), (&PA[09], None), (&PA[10], None)]);
    let rf233 = static_init!(ImixSubmodule,
                             ImixSubmodule::new(Some(rf233_detachable_pins), &PC[18]));

    let nrf_detachable_pins = static_init!([DetachablePin; 6],
                                           [(&PB[07], None),
                                            (&PA[17], None),
                                            (&PA[18], Some(A)),
                                            (&PC[07], Some(B)),
                                            (&PC[08], Some(B)),
                                            (&PC[09], None)]);

    let nrf = static_init!(ImixSubmodule,
                           ImixSubmodule::new(Some(nrf_detachable_pins), &PC[17]));

    let sensors = static_init!(ImixSubmodule, ImixSubmodule::new(None, &PC[16]));
    let trng = static_init!(ImixSubmodule, ImixSubmodule::new(None, &PC[19]));

    rf233.power(enabled_modules.rf233);
    nrf.power(enabled_modules.nrf51422);
    sensors.power(enabled_modules.sensors);
    trng.power(enabled_modules.trng);
}
