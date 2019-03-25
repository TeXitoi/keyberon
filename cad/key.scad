use <utils.scad>

module key(down=false) {
    translate([0,0,down?1:5]) linear_extrude(10, scale=0.7)
        rounded_square([18.5,18.5], r=3, center=true);
}

module switch() {
    translate([0,0,-5]) linear_extrude(6) rounded_square([14,14], r=1, center=true);
    linear_extrude(6, scale=0.8) rounded_square([15.6, 15.6], r=1, center=true);
    translate([0,0,-5]) cylinder(h=6, d=4, center=true);
}
