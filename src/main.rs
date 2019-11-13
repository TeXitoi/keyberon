#![no_main]
#![no_std]

extern crate panic_semihosting;

pub mod action;
pub mod debounce;
pub mod hid;
pub mod key_code;
pub mod keyboard;
pub mod layout;
pub mod matrix;

use crate::action::Action::{self, *};
use crate::action::{d, k, l, m};
use crate::debounce::Debouncer;
use crate::key_code::KeyCode::*;
use crate::keyboard::Keyboard;
use crate::matrix::{Matrix, PressedKeys};
use embedded_hal::digital::v2::{InputPin, OutputPin};
use generic_array::typenum::{U12, U5};
use rtfm::app;
use stm32_usbd::{UsbBus, UsbBusType};
use stm32f1xx_hal::gpio::{gpioa::*, gpiob::*, Input, Output, PullUp, PushPull};
use stm32f1xx_hal::prelude::*;
use stm32f1xx_hal::stm32;
use stm32f1xx_hal::{gpio, timer};
use usb_device::bus;
use usb_device::class::UsbClass;
use usb_device::prelude::*;
use void::Void;

type KeyboardHidClass = hid::HidClass<'static, UsbBusType, Keyboard<Leds>>;

pub struct Leds {
    caps_lock: gpio::gpioc::PC13<gpio::Output<gpio::PushPull>>,
}
impl keyboard::Leds for Leds {
    fn caps_lock(&mut self, status: bool) {
        if status {
            self.caps_lock.set_low().unwrap()
        } else {
            self.caps_lock.set_high().unwrap()
        }
    }
}

pub struct Cols(
    pub PB12<Input<PullUp>>,
    pub PB13<Input<PullUp>>,
    pub PB14<Input<PullUp>>,
    pub PB15<Input<PullUp>>,
    pub PA8<Input<PullUp>>,
    pub PA9<Input<PullUp>>,
    pub PA10<Input<PullUp>>,
    pub PB5<Input<PullUp>>,
    pub PB6<Input<PullUp>>,
    pub PB7<Input<PullUp>>,
    pub PB8<Input<PullUp>>,
    pub PB9<Input<PullUp>>,
);
impl_getter! {
    Cols,
    dyn InputPin<Error = Void>,
    U12,
    [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11]
}

pub struct Rows(
    pub PB11<Output<PushPull>>,
    pub PB10<Output<PushPull>>,
    pub PB1<Output<PushPull>>,
    pub PB0<Output<PushPull>>,
    pub PA7<Output<PushPull>>,
);
impl_getter! {
    Rows,
    dyn OutputPin<Error = Void>,
    U5,
    [0, 1, 2, 3, 4]
}

const CUT: Action = m(&[LShift, Delete]);
const COPY: Action = m(&[LCtrl, Insert]);
const PASTE: Action = m(&[LShift, Insert]);

#[rustfmt::skip]
pub static LAYERS: [[[Action; 12]; 5]; 2] = [
    [
        [k(Grave),   k(Kb1),k(Kb2),k(Kb3), k(Kb4),  k(Kb5),   k(Kb6),   k(Kb7),  k(Kb8), k(Kb9),  k(Kb0),   k(Minus)   ],
        [k(Tab),     k(Q),  k(W),  k(E),   k(R),    k(T),     k(Y),     k(U),    k(I),   k(O),    k(P),     k(LBracket)],
        [k(RBracket),k(A),  k(S),  k(D),   k(F),    k(G),     k(H),     k(J),    k(K),   k(L),    k(SColon),k(Quote)   ],
        [k(Equal),   k(Z),  k(X),  k(C),   k(V),    k(B),     k(N),     k(M),    k(Comma),k(Dot), k(Slash), k(Bslash)  ],
        [k(LCtrl),   l(1), k(LGui),k(LAlt),k(Space),k(LShift),k(RShift),k(Enter),k(RAlt),k(BSpace),k(Escape),k(RCtrl)  ],
    ], [
        [k(F1),      k(F2),    k(F3),k(F4),k(F5),k(F6),k(F7),k(F8),  k(F9),  k(F10), k(F11),  k(F12)   ],
        [k(Escape),  Trans,    Trans,Trans,Trans,Trans,Trans,Trans,  Trans,  Trans,  Trans,   k(PgUp)  ],
        [d(0),       d(1),     Trans,Trans,Trans,Trans,Trans,k(Left),k(Down),k(Up),  k(Right),k(PgDown)],
        [k(CapsLock),k(Delete),CUT,  COPY, PASTE,Trans,Trans,Trans,  Trans,  k(Home),k(Up),   k(End)   ],
        [Trans,      Trans,    Trans,Trans,Trans,Trans,Trans,Trans,  Trans,  k(Left),k(Down), k(Right) ],
    ]
];

// Generic keyboard from
// https://github.com/obdev/v-usb/blob/master/usbdrv/USB-IDs-for-free.txt
const VID: u16 = 0x27db;
const PID: u16 = 0x16c0;

#[app(device = stm32f1xx_hal::stm32)]
const APP: () = {
    static mut USB_DEV: UsbDevice<'static, UsbBusType> = ();
    static mut USB_CLASS: KeyboardHidClass = ();
    static mut MATRIX: Matrix<Cols, Rows> = ();
    static mut DEBOUNCER: Debouncer<PressedKeys<U5, U12>> = ();
    static mut LAYOUT: layout::Layout = layout::Layout::new(LAYERS);
    static mut TIMER: timer::Timer<stm32::TIM3> = ();

    #[init]
    fn init() -> init::LateResources {
        static mut USB_BUS: Option<bus::UsbBusAllocator<UsbBusType>> = None;

        let mut flash = device.FLASH.constrain();
        let mut rcc = device.RCC.constrain();

        let clocks = rcc
            .cfgr
            .use_hse(8.mhz())
            .sysclk(48.mhz())
            .pclk1(24.mhz())
            .freeze(&mut flash.acr);

        let mut gpioa = device.GPIOA.split(&mut rcc.apb2);
        let mut gpiob = device.GPIOB.split(&mut rcc.apb2);
        let mut gpioc = device.GPIOC.split(&mut rcc.apb2);

        let mut led = gpioc.pc13.into_push_pull_output(&mut gpioc.crh);
        led.set_high().unwrap();
        let leds = Leds { caps_lock: led };

        let usb_dm = gpioa.pa11;
        let usb_dp = gpioa.pa12.into_floating_input(&mut gpioa.crh);

        *USB_BUS = Some(UsbBus::new(device.USB, (usb_dm, usb_dp)));
        let usb_bus = USB_BUS.as_ref().unwrap();

        let usb_class = hid::HidClass::new(Keyboard::new(leds), &usb_bus);
        let usb_dev = UsbDeviceBuilder::new(usb_bus, UsbVidPid(VID, PID))
            .manufacturer("RIIR Task Force")
            .product("Keyberon")
            .serial_number(env!("CARGO_PKG_VERSION"))
            .build();

        let mut timer = timer::Timer::tim3(device.TIM3, 1.khz(), clocks, &mut rcc.apb1);
        timer.listen(timer::Event::Update);

        let mut matrix = matrix::Matrix::new(
            Cols(
                gpiob.pb12.into_pull_up_input(&mut gpiob.crh),
                gpiob.pb13.into_pull_up_input(&mut gpiob.crh),
                gpiob.pb14.into_pull_up_input(&mut gpiob.crh),
                gpiob.pb15.into_pull_up_input(&mut gpiob.crh),
                gpioa.pa8.into_pull_up_input(&mut gpioa.crh),
                gpioa.pa9.into_pull_up_input(&mut gpioa.crh),
                gpioa.pa10.into_pull_up_input(&mut gpioa.crh),
                gpiob.pb5.into_pull_up_input(&mut gpiob.crl),
                gpiob.pb6.into_pull_up_input(&mut gpiob.crl),
                gpiob.pb7.into_pull_up_input(&mut gpiob.crl),
                gpiob.pb8.into_pull_up_input(&mut gpiob.crh),
                gpiob.pb9.into_pull_up_input(&mut gpiob.crh),
            ),
            Rows(
                gpiob.pb11.into_push_pull_output(&mut gpiob.crh),
                gpiob.pb10.into_push_pull_output(&mut gpiob.crh),
                gpiob.pb1.into_push_pull_output(&mut gpiob.crl),
                gpiob.pb0.into_push_pull_output(&mut gpiob.crl),
                gpioa.pa7.into_push_pull_output(&mut gpioa.crl),
            ),
        );
        matrix.clear();

        init::LateResources {
            USB_DEV: usb_dev,
            USB_CLASS: usb_class,
            TIMER: timer,
            DEBOUNCER: Debouncer::new(PressedKeys::new(), PressedKeys::new(), 5),
            MATRIX: matrix,
        }
    }

    #[interrupt(priority = 2, resources = [USB_DEV, USB_CLASS])]
    fn USB_HP_CAN_TX() {
        usb_poll(&mut resources.USB_DEV, &mut resources.USB_CLASS);
    }

    #[interrupt(priority = 2, resources = [USB_DEV, USB_CLASS])]
    fn USB_LP_CAN_RX0() {
        usb_poll(&mut resources.USB_DEV, &mut resources.USB_CLASS);
    }

    #[interrupt(priority = 1, resources = [USB_CLASS, MATRIX, DEBOUNCER, LAYOUT, TIMER])]
    fn TIM3() {
        resources.TIMER.clear_update_interrupt_flag();

        if resources.DEBOUNCER.update(resources.MATRIX.get()) {
            let data = resources.DEBOUNCER.get();
            let mut report = key_code::KbHidReport::default();
            for kc in resources.LAYOUT.key_codes(data.iter_pressed()) {
                report.pressed(kc);
            }
            resources
                .USB_CLASS
                .lock(|k| k.device_mut().set_keyboard_report(report.clone()));
            while let Ok(0) = resources.USB_CLASS.lock(|k| k.write(report.as_bytes())) {}
        }
    }
};

fn usb_poll(usb_dev: &mut UsbDevice<'static, UsbBusType>, keyboard: &mut KeyboardHidClass) {
    if usb_dev.poll(&mut [keyboard]) {
        keyboard.poll();
    }
}
