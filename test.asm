TAG_SIZE=8
TAG_BITMASK=0xff
RECORD_SIZE=31
BLOCK_SIZE=8
BLOCK_MASK=0xff

TALLY_0=0
TALLY_1=4
TALLY_2=8
TALLY_3=12
BLOCK_ADDR=20
KEY_ADDR=16

# key is in format :

# rotr(r, bs) = ((bs << r) | (bs >> (BLOCK_SIZE - r))) & ((1 << BLOCK_SIZE) - 1)

# flip(bf) = ~bf
# shift(r, bs) = ((bs << r) | (bs >> (BLOCK_SIZE - r))) & ((1 << BLOCK_SIZE) - 1)
# swap(bx, by, px, py, s)

main:
    # set up stack (sp = r15 = 0x10000)
    li  r1, 1
    li  r2, 16
    shl r15, r1, r2

    # load addresses for bus access
    # 0x10000-0x10004 - 7-seg out
    # 0x10004-0x10008 - switch in
    # 0x10008 - btnC
    # 0x1000c - btnL
    # 0x10010 - btnR
    # 0x10014 - btnU
    # 0x10018 - btnD
    li  r1, 1
    li  r2, 16
    shl r1, r1, r2
    li  r2, 0
    or  r14, r1, r2
    li  r2, 4
    or  r13, r1, r2
    li  r2, 0x8
    or  r12, r1, r2

    # r14 - 7-seg output
    # r13 - switch input
    # r12 - btnC input

main__wait_load:
    lw  r1, r14, 0
    cmp r1, r0

    # Loop until the button is pressed
    beq main__wait_load

main__load_record:
    lw  r8, r13, 0

    # would be nice if this was a psuedoinstruction :p
    li  r1, main__load_record_ret1
    push r1
    jmp block_partition
main__load_record_ret1:

    jmp main__wait_load

block_partition:
    # Assume record in r8
    xor r1, r0, r0

    # r2 = shift_amt = RECORD_SIZE - BLOCK_SIZE
    li  r2, RECORD_SIZE
    li  r3, BLOCK_SIZE
    sub r2, r2, r3

block_partition__save_block:
    li  r4, BLOCK_MASK
    # r5 = block address
    li  r5, BLOCK_ADDR
    
    # block (r1) = (record >> shift_amt) & BLOCK_MASK
    shr r1, r8, r2
    and r1, r1, r4

    # mem[r5 + BLOCK_ADDR] = block
    sw r1, r5, 0

    # block_addr (r5) += 4
    li  r6, 4
    add r5, r5, r6

    # shift_amt -= BLOCK_SIZE
    sub r2, r2, r3
    
    # we dont need cmp as we can rely
    # on the sub instruction to get the info we need
    # i.e. if shift_amt was BLOCK_SIZE it will now be zero

    # if (shift_amt < 0) jmp block_partition__save_block_last
    blt block_partition__save_block_last

    # if (shift_amt == 0) jmp block_partition__save_block__end
    # TODO: Make sure flags gets held by alu on branch
    beq block_partition__save_block__end

block_partition__save_block_last:
    # Shift the record value left to add left padding bits

    # shift_amt = -shift_amt
    sub r2, r0, r2

    # r1 (block) = (record << shift_amt) & BLOCK_MASK
    shl r1, r8, r2
    and r1, r1, r4

    # mem[r5 + BLOCK_ADDR] = r1
    sw r1, r5, 0

block_partition__save_block__end:
    pop r1
    jmp r1

block_flip:
    # take block address in r8

    # block = (~block) & BLOCK_MASK
    lw  r1, r8, 0
    not r1, r1

    li  r2, BLOCK_MASK
    and r1, r1, r2

    sw  r1, r8, 0

    pop r1
    jmp r1

block_swap:
    # (s) = r1
    # (bx) = r8
    # (by) = r9
    # (px) = r10
    # (py) = r11
    
    # segment_mask = s - 1
    li  r2, 1

    #sub r1, r10, r11
    bge seg1_larger

seg1_smaller:
    sub r2, r0, r1
    li  r3, 1
    sub r2, r2, r3

    # shift_amt = px
    add r3, r10, 0

    jmp do_swap

seg1_larger:
    # segment_mask = (px - py) - 1
    li  r2, 1
    sub r2, r1, r2

    # shift_amt = py
    add r3, r11, 0

do_swap:
    shl r2, r2, r3

    # segx = bx & (segment_mask << shift_amt)
    and r4, r8, r2

    # segy = by & (segment_mask << shift_amt)
    and r5, r9, r2

    # segment_mask = ~segment_mask
    not r3, r3

    # bx = (bx & segment_mask) | segy
    and r8, r8, r3
    or  r8, r8, r5
    
    # by = (by & segment_mask) | segx
    and r9, r9, r3
    or  r9, r9, r4

block_swap__end:
    pop r1
    jmp r1

block_shift:
    # r8 = block
    # r1 = r

    li  r7, BLOCK_SIZE

    # ((bs << r) | (bs >> (BLOCK_SIZE - r))) & BLOCK_MASK
    
    # r2 = (bs << r)
    shl r2, r8, r1

    # r3 = (BLOCK_SIZE - r)
    sub r3, r7, r1
    # r3 = (bs >> (BLOCK_SIZE - r))
    shr r3, r8, r3 

    # r2 = ((bs << r) | (bs >> (BLOCK_SIZE - r)))
    or  r2, r2, r3

    li  r3, BLOCK_MASK
    and r2, r2, r3

    # Result in r8
    add r8, r2, r0
