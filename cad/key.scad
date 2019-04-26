use <utils.scad>

module keycap(down=false) {
  translate([0,0,down?1:5]) {
    difference() {
      hull() {
        linear_extrude(1, scale=0.1) square([18.3, 18.3], center=true);
        translate([0, 0, 7.4])
          rotate([180,0,0])
          linear_extrude(1, scale=0.1)
          rounded_square([12,12], r=1.5, center=true);
      }
      hull() {
        wall=1.6;
        translate([0,0,-0.1]) linear_extrude(1, scale=0.1)
          square([18.3-2*wall, 18.3-2*wall], center=true);
        translate([0, 0, 7.4-wall])
          rotate([180,0,0])
          linear_extrude(1, scale=0.1)
          square([(12-2*wall)/0.85,(12-2*wall)/0.85], center=true);
      }
    }
    difference() {
      cylinder(h=7, d=5.5);
      rotate([0,0,45]) cube([3.5, 3.5, 20], center=true);
    }
  }
}

module switch() {
    translate([0,0,-5]) linear_extrude(6) rounded_square([13.5,13.5], r=1, center=true);
    linear_extrude(6, scale=0.8) rounded_square([15.6, 15.6], r=1, center=true);
    translate([0,0,-5]) cylinder(h=6, d=4, center=true);
}

keycap();
