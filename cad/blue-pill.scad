use <utils.scad>

pin_size = 2.54;
depth=1.5;

module pin(with_dupond=false) {
  // plastic cube
  color([0.2, 0.2, 0.2]) translate([0, 0, pin_size/2])
    cube([pin_size, pin_size, pin_size], center=true);

  // pin
  color([0.7, 0.7, 0.7]) translate([0, 0, pin_size])
    cube([pin_size/5, pin_size/5, pin_size*4], center=true);

  // dupond
  if (with_dupond) {
    color([0.2, 0.2, 0.2]) translate([0, 0, 3.6*pin_size])
      cube([pin_size, pin_size, 5*pin_size], center=true);
    color([0.4, 0, 0.4]) translate([0, 0, 6*pin_size])
      cylinder(d=1, h=10);
  }
}

module blue_pill(boot_pins=true) {
  width=52.8;
  height=22.5;

  difference() {
    // PCB
    color([0, 0, 0.6])
      cube([width, height, depth], center=true);

    // holes
    for (i = [-3, 3])
      for (j = [0:19])
        translate([-pin_size * 19 / 2 + j * pin_size, i * pin_size, 0])
          cylinder(d=1, h=2*depth, center=true);
  }

  // MCU
  color([0.2, 0.2, 0.2])
    rotate([0, 0, 45])
      translate([0, 0, depth])
        cube([6.7, 6.7, depth], center=true);

  // USB
  color([0.7, 0.7, 0.7])
    translate([-(width-5.4)/2, 0, 2.6/2+depth/2])
      cube([5.4, 7.5, 2.6], center=true);

  // debugger
  for (i = [-1.5:1.5])
    translate([width / 2 - 4.7, i * pin_size, depth/2 + pin_size/2])
      rotate([0,90,0])
        pin(with_dupond=false);

  // boot pins
  if (boot_pins) {
  for (i=[0.5:1.5])
    for (j=[-6.5:-4.5])
      translate([j*pin_size, i*pin_size, depth/2])
        pin();
  }

  // HSE
  color([0.7, 0.7, 0.7]) translate([3.5*pin_size,0,depth/2]) linear_extrude(3)
    rounded_square([3.5,10], r=1, center=true);

  // LSE
  color([0.2, 0.2, 0.2]) translate([5.5*pin_size,0,depth/2+2.4/2])
    cube([3, 7.8, 2.4], center=true);

  // power led
  color([0.8, 0, 0]) translate([7*pin_size,3,depth/2+1/2])
    cube([1,1,1], center=true);

  // user led
  color([0, 0.8, 0]) translate([7*pin_size,-3,depth/2+1/2])
    cube([1,1,1], center=true);

  // reset
  color([0.9, 0.9, 0.9]) translate([-5.5*pin_size,-1.5*pin_size,depth/2+3.5/2]) {
    cube([6,3.5,3.5], center=true);
    cube([3,1.4,3.6+0.6], center=true);
  }
}

module blue_pill_pocket(under=2, over=3, open_under=false, open_over=false, led_holes=false) {
  difference() {
    union() {
      // PCB
      cube([53.5, 23, 2], center=true);

      // debugger
      translate([33, 0, (depth+pin_size)/2]) cube([25, 11, 3], center=true);

      // usb
      translate([-53.5/2-1.25-30/2, 0, (depth+pin_size)/2]) {
        cube([30, 11, 9], center=true);
        cube([35, 7.5, 2.5], center=true);
      }

      // under
      translate([0, 0, -(under+1)/2])
        cube([53.5, 23-(open_under ? 0 : 4), under+1], center=true);

      // over
      translate([0, 0, (over+1)/2])
        cube([53.5, 23-(open_over ? 0 : 4), over+1], center=true);

      // HSE
      translate([3.5*pin_size,0,depth/2]) linear_extrude(3.6)
          rounded_square([4.5,11], r=1, center=true);

      // reset
      translate([-5.5*pin_size,-1.5*pin_size,depth/2+3.6/2]) {
        cube([7,4.5,3.6], center=true);
        cube([4,2.5,3.6+4], center=true);
      }

      // led_holes
      if (led_holes) {
        for (x=[-3,3]) {
          translate([7*pin_size,x,0]) cylinder(d=1.5, h=20);
        }
      }
    }
    if (led_holes) {
      translate([7*pin_size,0,over+1]) cube([3, 0.4, over*2], center=true);
    }
  }
}

blue_pill(boot_pins=false);
#blue_pill_pocket();
