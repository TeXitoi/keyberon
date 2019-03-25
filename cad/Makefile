all: left-case.stl right-case.stl left-back.stl right-back.stl key-switch-up.stl key-switch-down.stl

%.stl: %.printing.scad
	openscad -o $@ $<

left-case.stl: left-case.printing.scad case.scad printing.scad utils.scad
right-case.stl: right-case.printing.scad case.scad printing.scad utils.scad
left-back.stl: left-case.printing.scad case.scad printing.scad utils.scad
right-back.stl: right-case.printing.scad case.scad printing.scad utils.scad
key-switch-down.stl: key-switch-down.printing.scad key.scad printing.scad utils.scad
key-switch-up.stl: key-switch-up.printing.scad key.scad printing.scad utils.scad
