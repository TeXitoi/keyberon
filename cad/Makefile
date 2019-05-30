all: case.stl back.stl key-switch-up.stl key-switch-down.stl keycap.stl

%.stl: %.printing.scad
	openscad -o $@ $<

case.stl: case.printing.scad case.scad printing.scad utils.scad blue-pill.scad
back.stl: case.printing.scad case.scad printing.scad utils.scad blue-pill.scad
key-switch-down.stl: key-switch-down.printing.scad key.scad printing.scad utils.scad
key-switch-up.stl: key-switch-up.printing.scad key.scad printing.scad utils.scad
keycap.stl: keycap.printing.scad key.scad printing.scad utils.scad
