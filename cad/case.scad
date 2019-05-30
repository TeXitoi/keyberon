use <utils.scad>
use <blue-pill.scad>
use <key.scad>
include <printing.scad>

nb_row=5;
nb_col=12;
rounding=3;
border=8;
switch_hole=14;
inter_switch=19.05;
back_thickness=1.4;

case_depth=8+back_thickness;
case_width=inter_switch*nb_col-(inter_switch-switch_hole)+border*2;
case_height=inter_switch*nb_row-(inter_switch-switch_hole)+border*2;
bp_width=23;
bp_height=53.5;
bp_x=case_width/2+5.5;
bp_y=case_height/2-53.5/2-1.5;
mcu_width=bp_x-case_width/2+bp_width/2+border;
mcu_height=60;
cut_offset= nb_col%2==0 ? 0 : inter_switch/2;

module key_placement() {
    for (i=[0:nb_col-1]) {
        for (j=[0:nb_row-1]) {
            translate([inter_switch*((nb_col-1)/2-i), inter_switch*(j-(nb_row-1)/2), 0]) {
                children();
            }
        }
    }
}

module bp_placement() {
  translate([bp_x, bp_y, 1+3+1]) rotate([180,0,-90]) children();
}

module hole_placement() {
    b=5;
    for (coord=[[ b-case_width/2,            b-case_height/2,            case_depth-back_thickness],
                [ b-case_width/2,           -b+case_height/2,            case_depth-back_thickness],
                [-b+case_width/2,            b-case_height/2,            case_depth-back_thickness],
                [-b+case_width/2+mcu_width, -b+case_height/2,            case_depth-back_thickness],
                [-b+case_width/2+mcu_width,  b+case_height/2-mcu_height, case_depth-back_thickness],
                [cut_offset,                -b+case_height/2,            case_depth-back_thickness],
                [cut_offset,                 b-case_height/2,            case_depth-back_thickness]])
    {
        translate(coord) children();
    }
}

module wire_hole(epsilon=0) {
    translate([bp_x-bp_width/2, bp_y, (case_depth-4-back_thickness+epsilon)/2+4])
        cube([bp_width, bp_height-2*border+3, case_depth-4-back_thickness+epsilon], center=true);
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
        translate([0,0,case_depth/2+4])
            cube([case_width-2*border, case_height-2*border, case_depth], center=true);

        // backpanel pocket
        translate([0,0,case_depth-back_thickness]) linear_extrude(2*back_thickness)
            rounded_square([case_width-2, case_height-2], r=rounding-1, center=true);
        translate([case_width/2, case_height/2-mcu_height/2,case_depth-back_thickness])
          linear_extrude(2*back_thickness)
          rounded_square([mcu_width*2-2, mcu_height-2], r=rounding-1, center=true);

        // bp hole
        bp_placement() blue_pill_pocket(under=4, open_under=true, led_holes=true);
        wire_hole(epsilon=1);

        // switch holes
        key_placement() {
            translate([0,0,5]) {
                cube([switch_hole,switch_hole,15], center=true);
                translate([0,0,1.5]) cube([5, switch_hole+3, 10], center=true);
            }
        }

        // screw holes
        hole_placement() {
            cylinder(d=1.8, h=4*2, center=true);
        }
    }
}

module back() {
    difference() {
        union() {
            translate([0,0,case_depth-back_thickness]) linear_extrude(back_thickness)
              rounded_square([case_width-3, case_height-3], r=rounding-1.5, center=true);
            translate([case_width/2, case_height/2-mcu_height/2,case_depth-back_thickness])
              linear_extrude(back_thickness)
              rounded_square([mcu_width*2-3, mcu_height-3], r=rounding-1.5, center=true);
            for (i=[-1, 1]) {
                translate([bp_x+i*(bp_width/2-1.5/2-1.5), bp_y, (case_depth-6)/2+6])
                    cube([1.5, bp_height-1, case_depth-6], center=true);
            }
        }
        wire_hole();
        hole_placement() {
            translate([0,0,-1]) cylinder(r1=0.5, r2=3+0.5, h=3);
        }
    }
}

module switches() {
    key_placement() {
        color([1,1,1,0.8]) {
            rotate([180,0,0]) switch();
        }
    }
}

module keys() {
    for (i=[0:nb_col-1]) {
        for (j=[0:nb_row-1]) {
            translate([19*((nb_col-1)/2-i), 19*(j-(nb_row-1)/2), 0]) {
                note=(i*4+j*3+10)%12;
                c = note==1||note==3||note==6||note==8||note==10 ? [0.2,0.2,0.2] : [0.9,0.9,0.9];
                color(c) rotate([180,0,0]) keycap();
            }
        }
    }
}

color([0.3,0.3,0.3]) {
    case();
    back();
}
bp_placement() blue_pill(boot_pins=false);
switches();
keys();
