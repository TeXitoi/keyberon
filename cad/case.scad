use <utils.scad>
use <blue-pill.scad>
use <key.scad>

rounding=3;
border=5;
case_width=19*12-5+border*2;
case_height=19*5-5+border*2;
case_depth=9;
bp_width=23;
bp_height=53.5;
bp_x=case_width/2+23/2-23/2+5.5+2;
bp_y=case_height/2-53.5/2-1.5;
mcu_width=bp_x-case_width/2++bp_width/2+border;
mcu_height=58;

module key_placement() {
    for (i=[0:11]) {
        for (j=[0:4]) {
            translate([19*(5.5-i), 19*(j-2), 0]) {
                children();
            }
        }
    }
}

module case() {
    difference() {
        union() {
            linear_extrude(case_depth)
                rounded_square([case_width, case_height], r=rounding, center=true);

            translate([case_width/2, case_height/2-mcu_height/2,0]) linear_extrude(case_depth)
                rounded_square([mcu_width*2, mcu_height], r=rounding, center=true);
        }

        // back hole
        translate([0,0,case_depth/2+4]) cube([19*12-5, 19*5-5, case_depth], center=true);

        // backpanel pocket
        translate([0,0,case_depth-1]) linear_extrude(2)
            rounded_square([case_width-2, case_height-2], r=rounding-1, center=true);
        translate([case_width/2, case_height/2-mcu_height/2,case_depth-1]) linear_extrude(2)
            rounded_square([mcu_width*2-2, mcu_height-2], r=rounding-1, center=true);

        // bp hole
        translate([bp_x, bp_y, case_depth/2+1]) cube([bp_width-4, bp_height, case_depth], center=true);
        translate([bp_x, bp_y, case_depth/2+4]) cube([bp_width, bp_height, case_depth], center=true);
        translate([bp_x-bp_width/2, bp_y, case_depth/2+4]) cube([bp_width, bp_height-2*border+3, case_depth], center=true);

        // debugger hole
        translate([bp_x, bp_y-bp_height/2, 5-(1.6+2.6)/2]) cube([11, 25, 3], center=true);

        // usb hole
        translate([bp_x, bp_y+bp_height/2+5, 5-(1.6+2.6)/2]) cube([12, 10.01, 8], center=true);

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

module back() {
    translate([0,0,case_depth-1]) linear_extrude(1)
        rounded_square([case_width-3, case_height-3], r=rounding-1.5, center=true);
    translate([case_width/2, case_height/2-mcu_height/2,case_depth-1]) linear_extrude(1)
        rounded_square([mcu_width*2-3, mcu_height-3], r=rounding-1.5, center=true);

}

color([0.3,0.3,0.3])
case();

color([0.3,0.3,0.3])
back();

translate([bp_x, bp_y, 1+3+1]) rotate([180,0,-90]) blue_pill();

//switches
key_placement() {
    color([1,1,1,0.8]) {
        cube([14,14,10], center=true);
        translate([0,0,5]) cylinder(h=6, d=4, center=true);
    }
 }

// keys
for (i=[0:11]) {
    for (j=[0:4]) {
        translate([19*(5.5-i), 19*(j-2), 0]) {
            note=(i*4+j*3+10)%12;
            c = note==1||note==3||note==6||note==8||note==10 ? [0.2,0.2,0.2] : [0.9,0.9,0.9];
            color(c) translate([0,0,-5]) rotate([180,0,0]) key();
        }
    }
}
