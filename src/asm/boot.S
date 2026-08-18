# boot.S
# S-mode entry point, entered from OpenSBI.
# shimon daniel
# 18 August 2026
#
# OpenSBI hands control over with:
#   a0 = hartid of the current hart
#   a1 = physical address of the device tree blob
#   satp = 0 (no paging), interrupts disabled

.option norvc
.section .text.init
.global _start
_start:
	# Only hart 0 boots; every other hart parks.
	bnez	a0, park

.option push
.option norelax
	la		gp, _global_pointer
.option pop

	# Zero the .bss section
	la		t0, _bss_start
	la		t1, _bss_end
	bgeu	t0, t1, 2f
1:
	sd		zero, (t0)
	addi	t0, t0, 8
	bltu	t0, t1, 1b
2:
	# Boot stack
	la		sp, _stack_top

	# Keep interrupts off until there is a trap handler
	csrw	sie, zero
	csrci	sstatus, 2		# clear SSTATUS.SIE

	# a0/a1 are still hartid/dtb -> passed straight to kmain
	j		kmain

park:
	wfi
	j		park
