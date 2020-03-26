#![no_main]
#![no_std]

use core::convert::Infallible;
use embedded_hal::digital::v2::{InputPin, OutputPin};
use generic_array::typenum::{U12, U4};
use keyberon::action::Action::{self, *};
use keyberon::action::{d, k, l, m};
use keyberon::debounce::Debouncer;
use keyberon::impl_heterogenous_array;
use keyberon::key_code::KeyCode::*;
use keyberon::key_code::{KbHidReport, KeyCode};
use keyberon::layout::Layout;
use keyberon::matrix::{Matrix, PressedKeys};
use panic_semihosting as _;
use rtfm::app;
use stm32f1xx_hal::gpio::{gpioa::*, gpiob::*, Input, Output, PullUp, PushPull};
use stm32f1xx_hal::prelude::*;
use stm32f1xx_hal::usb::{Peripheral, UsbBus, UsbBusType};
use stm32f1xx_hal::{gpio, pac, timer};
use usb_device::bus::UsbBusAllocator;
use usb_device::class::UsbClass as _;

type UsbClass = keyberon::Class<'static, UsbBusType, Leds>;
type UsbDevice = keyberon::Device<'static, UsbBusType>;

pub struct Leds {
	caps_lock: gpio::gpioc::PC13<gpio::Output<gpio::PushPull>>,
}
impl keyberon::keyboard::Leds for Leds {
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
impl_heterogenous_array! {
	Cols,
	dyn InputPin<Error = Infallible>,
	U12,
	[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11]
}

pub struct Rows(
	pub PB11<Output<PushPull>>,
	pub PB10<Output<PushPull>>,
	pub PB1<Output<PushPull>>,
	pub PB0<Output<PushPull>>,
);
impl_heterogenous_array! {
	Rows,
	dyn OutputPin<Error = Infallible>,
	U4,
	[0, 1, 2, 3]
}

const CUT: Action = m(&[LShift, Delete]);
const COPY: Action = m(&[LCtrl, Insert]);
const PASTE: Action = m(&[LShift, Insert]);
const C_ENTER: Action = HoldTap {
	timeout: 200,
	hold: &k(LCtrl),
	tap: &k(Enter),
};
const L1_SP: Action = HoldTap {
	timeout: 200,
	hold: &l(1),
	tap: &k(Space),
};

#[rustfmt::skip]
pub static LAYERS: keyberon::layout::Layers = &[
	&[
		&[k(Escape)	,k(Q)			,k(W)	,k(E)		,k(R)	,k(T)	,k(Y),     k(U)	,k(I)	,k(O)		,k(P)		,k(LBracket)],
		&[k(Tab)	,k(A)			,k(S)	,k(D)		,k(F)	,k(G)	,k(H),     k(J)	,k(K)	,k(L)		,k(SColon)	,k(Quote)   ],
		&[k(LShift)	,k(NonUsBslash)	,k(Z)	,k(X)		,k(C)	,k(V)	,k(B),     k(N)	,k(M)	,k(Comma)	,k(Dot)		,k(Slash)	],
		&[Trans		,Trans			,k(LAlt),k(LGui)	,C_ENTER,l(2)	,k(BSpace),L1_SP,k(RAlt),Trans		,Trans		,Trans		],
	], &[
		&[k(Grave)	,Trans	,Trans		,Trans	,Trans		,k(PgUp)			,k(PgDown)		,k(Home),k(Up)	,k(End)		,k(Minus)	,k(Equal)	],
		&[Trans		,Trans	,Trans		,Trans	,Trans		,k(MediaVolDown)	,k(MediaVolUp)	,k(Left),k(Down),k(Right)	,Trans		,k(RBracket)],
		&[Trans		,k(Kb1)	,k(Kb2)		,k(Kb3)	,k(Kb4)		,k(Kb5)				,k(Kb6)			,k(Kb7)	,k(Kb8)	,k(Kb9)		,k(Kb0)		,k(Bslash)	],
		&[Trans		,Trans	,Trans		,Trans	,Trans		,Trans				,Trans			,Trans	,Trans	,Trans		,Trans		,Trans		],
	], &[
		&[Trans		,k(F1)	,k(F2)	,k(F3)	,k(F4)	,Trans	,Trans		,m(&[RAlt, Kb8])	,m(&[RAlt, Kb9])	,Trans				,Trans	,Trans ],
		&[Trans		,k(F5)	,k(F6)	,k(F7)	,k(F8)	,Trans	,Trans		,m(&[RAlt, Kb7])	,m(&[RAlt, Kb0])	,Trans				,Trans	,Trans ],
		&[Trans		,k(F9)	,k(F10)	,k(F11)	,k(F12)	,Trans	,Trans		,Trans				,m(&[LShift, Kb8])	,m(&[LShift, Kb9])	,Trans	,Trans ],
		&[Trans		,Trans	,Trans	,Trans	,Trans	,Trans	,Trans		,Trans				,Trans				,Trans				,Trans	,Trans ],
	],
];

#[app(device = stm32f1xx_hal::pac, peripherals = true)]
const APP: () = {
	struct Resources {
		usb_dev: UsbDevice,
		usb_class: UsbClass,
		matrix: Matrix<Cols, Rows>,
		debouncer: Debouncer<PressedKeys<U4, U12>>,
		layout: Layout,
		timer: timer::CountDownTimer<pac::TIM3>,
	}

	#[init]
	fn init(c: init::Context) -> init::LateResources {
		static mut USB_BUS: Option<UsbBusAllocator<UsbBusType>> = None;

		let mut flash = c.device.FLASH.constrain();
		let mut rcc = c.device.RCC.constrain();

		let clocks = rcc
			.cfgr
			.use_hse(8.mhz())
			.sysclk(48.mhz())
			.pclk1(24.mhz())
			.freeze(&mut flash.acr);

		let mut gpioa = c.device.GPIOA.split(&mut rcc.apb2);
		let mut gpiob = c.device.GPIOB.split(&mut rcc.apb2);
		let mut gpioc = c.device.GPIOC.split(&mut rcc.apb2);

		let mut led = gpioc.pc13.into_push_pull_output(&mut gpioc.crh);
		led.set_high().unwrap();
		let leds = Leds { caps_lock: led };

		let usb_dm = gpioa.pa11;
		let usb_dp = gpioa.pa12.into_floating_input(&mut gpioa.crh);

		let usb = Peripheral {
			usb: c.device.USB,
			pin_dm: usb_dm,
			pin_dp: usb_dp,
		};

		*USB_BUS = Some(UsbBus::new(usb));
		let usb_bus = USB_BUS.as_ref().unwrap();

		let usb_class = keyberon::new_class(usb_bus, leds);
		let usb_dev = keyberon::new_device(usb_bus);

		let mut timer =
			timer::Timer::tim3(c.device.TIM3, &clocks, &mut rcc.apb1).start_count_down(1.khz());
		timer.listen(timer::Event::Update);

		let matrix = Matrix::new(
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
			),
		);

		init::LateResources {
			usb_dev,
			usb_class,
			timer,
			debouncer: Debouncer::new(PressedKeys::default(), PressedKeys::default(), 5),
			matrix: matrix.unwrap(),
			layout: Layout::new(LAYERS),
		}
	}

	#[task(binds = USB_HP_CAN_TX, priority = 2, resources = [usb_dev, usb_class])]
	fn usb_tx(mut c: usb_tx::Context) {
		usb_poll(&mut c.resources.usb_dev, &mut c.resources.usb_class);
	}

	#[task(binds = USB_LP_CAN_RX0, priority = 2, resources = [usb_dev, usb_class])]
	fn usb_rx(mut c: usb_rx::Context) {
		usb_poll(&mut c.resources.usb_dev, &mut c.resources.usb_class);
	}

	#[task(binds = TIM3, priority = 1, resources = [usb_class, matrix, debouncer, layout, timer])]
	fn tick(mut c: tick::Context) {
		c.resources.timer.clear_update_interrupt_flag();

		for event in c
			.resources
			.debouncer
			.events(c.resources.matrix.get().unwrap())
		{
			send_report(c.resources.layout.event(event), &mut c.resources.usb_class);
		}
		send_report(c.resources.layout.tick(), &mut c.resources.usb_class);
	}
};

fn send_report(iter: impl Iterator<Item = KeyCode>, usb_class: &mut resources::usb_class<'_>) {
	use rtfm::Mutex;
	let report: KbHidReport = iter.collect();
	if usb_class.lock(|k| k.device_mut().set_keyboard_report(report.clone())) {
		while let Ok(0) = usb_class.lock(|k| k.write(report.as_bytes())) {}
	}
}

fn usb_poll(usb_dev: &mut UsbDevice, keyboard: &mut UsbClass) {
	if usb_dev.poll(&mut [keyboard]) {
		keyboard.poll();
	}
}
