use <utils.scad>

module keycap_shape(d1=18.3, d2=12, h=7.5, r=1.5) {
  hull() {
    linear_extrude(1, scale=0.1) square([d1, d1], center=true);
    translate([0, 0, h])
      rotate([180,0,0])
      linear_extrude(1, scale=0.1)
      rounded_square([d2,d2], r=r, center=true);
  }
}

module keycap(down=false) {
  w=1.2;
  translate([0,0,down?1:5]) {
    difference() {
      keycap_shape(d1=18.3, d2=12, h=7.5, r=1.5);
      intersection() {
        translate([0,0,-0.01]) keycap_shape(d1=18.3-2*w, d2=12-2*w, h=7.5, r=0);
        cube([20, 20, (7.5-w)*2], center=true);
      }
    }
    difference() {
      cylinder(h=7, d=5.5);
      rotate([0,0,45]) cube([3.3, 3.3, 20], center=true);
    }
  }
}

module switch() {
    translate([0,0,-5]) linear_extrude(6) rounded_square([13.5,13.5], r=1, center=true);
    linear_extrude(6, scale=0.8) rounded_square([15.6, 15.6], r=1, center=true);
    translate([0,0,-5]) cylinder(h=6, d=4, center=true);
}

keycap();
