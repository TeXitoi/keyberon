use <utils.scad>
use <blue-pill.scad>

rounding=3;
case_width=19*12+5;
case_height=19*5+5;
mcu_width=27;
mcu_height=58;
bp_width=23;
bp_height=53.5;
bp_x=case_width/2+23/2;
bp_y=case_height/2-53.5/2-1.5;

module case() {
    difference() {
        union() {
            linear_extrude(8)
                rounded_square([case_width, case_height], r=rounding, center=true);

            translate([case_width/2, case_height/2-mcu_height/2,0]) linear_extrude(8)
                rounded_square([mcu_width*2, mcu_height], r=rounding, center=true);
        }

        // backpanel pocket
        translate([0,0,7]) linear_extrude(2)
            rounded_square([case_width-2, case_height-2], r=rounding-1, center=true);
        translate([case_width/2, case_height/2-mcu_height/2,7]) linear_extrude(2)
            rounded_square([mcu_width*2-2, mcu_height-2], r=rounding-1, center=true);

        // bp hole
        translate([bp_x, bp_y, 8]) cube([bp_width, bp_height, 8], center=true);
        translate([bp_x, bp_y, 6]) cube([bp_width-4, bp_height, 10], center=true);
        translate([bp_x-bp_width/2, bp_y, 9]) cube([bp_width, bp_height-8, 10], center=true);

        // debugger hole
        translate([bp_x, bp_y-bp_height/2, 5-(1.6+2.6)/2]) cube([11, 25, 3], center=true);

        // usb hole
        translate([bp_x, bp_y+bp_height/2+5, 5-(1.6+2.6)/2]) cube([12, 10.01, 8], center=true);

        // back hole
        translate([0,0,5+4]) cube([19*12-5, 19*5-5, 10], center=true);

        // switch holes
        for (i=[0:11]) {
            for (j=[0:4]) {
                translate([19*(i-5.5), 19*(j-2), 5]) {
                    cube([14,14,15], center=true);
                    translate([0,0,1.5]) cube([5, 14+3, 10], center=true);
                }
            }
        }
    }
}

color([0.3,0.3,0.3])
case();

translate([bp_x, bp_y, 1+3+1]) rotate([180,0,-90]) blue_pill();
