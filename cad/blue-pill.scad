module pin(with_dupond=false) {
  pin = 2.54;

  // plastic cube
  color([0.2, 0.2, 0.2]) translate([0, 0, pin/2])
    cube([pin, pin, pin], center=true);

  // pin
  color([0.7, 0.7, 0.7]) translate([0, 0, pin])
    cube([pin/5, pin/5, pin*4], center=true);

  // dupond
  if (with_dupond) {
    color([0.2, 0.2, 0.2]) translate([0, 0, 3.6*pin])
      cube([pin, pin, 5*pin], center=true);
    color([0.4, 0, 0.4]) translate([0, 0, 6*pin])
      cylinder(d=1, h=10);
  }
}

module blue_pill() {
  width=52.8;
  height=22.5;
  depth=1.4;
  pin=2.54;

  difference() {
    // PCB
    color([0, 0, 0.6])
      cube([width, height, depth], center=true);

    // holes
    for (i = [-3, 3])
      for (j = [0:19])
        translate([-pin * 19 / 2 + j * pin, i * pin, 0])
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
    translate([width / 2 - 4.7, i * pin, depth/2 + pin/2])
      rotate([0,90,0])
        pin(with_dupond=false);

  // boot pins
  /*
  for (i=[0.5:1.5])
    for (j=[-6.5:-4.5])
      translate([j*pin, i*pin, depth/2])
        pin();
  */
}

blue_pill();
