#!/bin/bash

screenshot_tests=(
  'blargg_nes_cpu_test5/cpu'
  'blargg_nes_cpu_test5/official'
  'cpu_dummy_reads/cpu_dummy_reads'
  'cpu_dummy_writes/cpu_dummy_writes_oam'
  'cpu_dummy_writes/cpu_dummy_writes_ppumem'
)

blargg_tests=(
  #'cpu_exec_space/test_cpu_exec_space_apu'
  #'cpu_exec_space/test_cpu_exec_space_ppuio'
  #'cpu_interrupts_v2/cpu_interrupts'
  #'cpu_interrupts_v2/rom_singles/1-cli_latency'
  #'cpu_interrupts_v2/rom_singles/2-nmi_and_brk'
  #'cpu_interrupts_v2/rom_singles/3-nmi_and_irq'
  #'cpu_interrupts_v2/rom_singles/4-irq_and_dma'
  #'cpu_interrupts_v2/rom_singles/5-branch_delays_irq'
  #'instr_misc/instr_misc'
  #'instr_test-v3/rom_singles/01-implied'
  #'instr_test-v3/rom_singles/02-immediate'
  #'instr_test-v3/rom_singles/03-zero_page'
  #'instr_test-v3/rom_singles/04-zp_xy'
  #'instr_test-v3/rom_singles/05-absolute'
  #'instr_test-v3/rom_singles/06-abs_xy'
  #'instr_test-v3/rom_singles/07-ind_x'
  #'instr_test-v3/rom_singles/08-ind_y'
  #'instr_test-v3/rom_singles/09-branches'
  #'instr_test-v3/rom_singles/10-stack'
  #'instr_test-v3/rom_singles/11-jmp_jsr'
  #'instr_test-v3/rom_singles/12-rts'
  #'instr_test-v3/rom_singles/13-rti'
  #'instr_test-v3/rom_singles/14-brk'
  #'instr_test-v3/rom_singles/15-special'
  #'instr_test-v3/all_instrs'
  #'instr_test-v3/official_only'
  #'instr_timing/instr_timing'
)

for i in "${screenshot_tests[@]}"
do
	echo ""
	echo "##### $i with screenshot #####"
	target/release/rusticnes-cli cart "../nes-test-roms/$i.nes" frames 1000 screenshot "../nes-test-roms/$i.png"
done

for i in "${blargg_tests[@]}"
do
	echo ""
	echo "##### $i as blargg #####"
	target/release/rusticnes-cli cart "../nes-test-roms/$i.nes" frames 1000 blargg "../nes-test-roms/$i.txt"
done


