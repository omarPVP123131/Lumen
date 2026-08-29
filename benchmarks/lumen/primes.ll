; ModuleID = 'lumen'
source_filename = "lumen.nv"
target datalayout = "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-i128:128-f80:128-n8:16:32:64-S128"

declare i64 @_lw_int(i64)
declare i64 @_lw_flt(double)
declare i64 @_lw_bool(i64)
declare i64 @_lw_str(i64)
declare i64 @_lw_void()
declare i64 @_lw_none()
declare void @_lw_print(i64)
declare void @_lw_print_blank()
declare i64 @_lw_join(i64, i64)
declare i64 @_lw_bin(i64, i64, i64)
declare i64 @_lw_un(i64, i64)
declare i64 @_lw_truthy_i(i64)
declare i64 @_lw_arr_new()
declare i64 @_lw_arr_push(i64, i64)
declare i64 @_lw_arr_get(i64, i64)
declare i64 @_lw_arr_set(i64, i64, i64)
declare i64 @_lw_arr_len(i64)
declare i64 @_lw_arr_rev(i64)
declare i64 @_lw_arr_sort(i64)
declare i64 @_lw_st_new()
declare i64 @_lw_st_add(i64, i64, i64)
declare i64 @_lw_st_get(i64, i64)
declare i64 @_lw_st_set(i64, i64, i64)
declare i64 @_lw_tup_new()
declare i64 @_lw_tup_push(i64, i64)
declare i64 @_lw_tup_get(i64, i64)
declare i64 @_lw_read()
declare i64 @_lw_typeof(i64)
declare i64 @_lw_to_text(i64)
declare i64 @_lw_sub(i64, i64, i64)
declare i64 @_lw_concat_list(i64)
declare i64 @_lw_some(i64)
declare i64 @_lw_ok(i64)
declare i64 @_lw_err(i64)
declare i64 @_lw_map_new()
declare i64 @_lw_map_set(i64, i64, i64)
declare i64 @_lw_map_get(i64, i64)
declare i64 @_lw_map_has(i64, i64)
declare i64 @_lw_map_len(i64)
declare i64 @_lw_map_keys(i64)
declare void @_lw_try_begin()
declare void @_lw_try_end()
declare i64 @_lw_err_active()
declare i64 @_lw_err_take()
declare i64 @_lw_kind(i64)
declare i64 @_lw_payload(i64)
declare i64 @_lw_enm_new(i64, i64, i64)
declare i64 @_lw_enm_variant_is(i64, i64)
declare i64 @_lw_fref(i64, i64)
declare i64 @_lw_fref_addr(i64)
declare i64 @_lw_mkref(i64)
declare i64 @_lw_load_slot(i64)
declare void @_lw_store_slot(i64, i64)
declare void @_lw_store_slot_direct(i64, i64)
declare i64 @_lw_dcp(i64)
declare i64 @_lw_arr_push_ip(i64, i64)
declare i64 @_lw_abs(i64)
declare i64 @_lw_sqrt(i64)
declare i64 @_lw_pow(i64, i64)
declare i64 @_lw_floor(i64)
declare i64 @_lw_ceil(i64)
declare i64 @_lw_round(i64)

@lw_str_0 = private unnamed_addr constant [8 x i8] c"primes:\00"

define i64 @lum___main__() {
entry:
  %r0 = call i64 @_lw_void()
  %r1 = getelementptr [8 x i8], [8 x i8]* @lw_str_0, i64 0, i64 0
  %r2 = ptrtoint i8* %r1 to i64
  %r3 = call i64 @_lw_str(i64 %r2)
  %r4 = call i64 @_lw_int(i64 20000)
  %r5 = call i64 @_lw_dcp(i64 %r4)
  %r6 = call i64 @lum_contar_primos(i64 %r5)
  %r7 = call i64 @_lw_err_active()
  %r8 = icmp ne i64 %r7, 0
  br i1 %r8, label %ec_d_9, label %ec_o_10
ec_d_9:
  %r11 = call i64 @_lw_void()
  ret i64 %r11
ec_o_10:
  %r12 = call i64 @_lw_bin(i64 1, i64 %r3, i64 %r6)
  %r13 = call i64 @_lw_err_active()
  %r14 = icmp ne i64 %r13, 0
  br i1 %r14, label %ec_d_15, label %ec_o_16
ec_d_15:
  %r17 = call i64 @_lw_void()
  ret i64 %r17
ec_o_16:
  call void @_lw_print(i64 %r12)
  %r18 = call i64 @_lw_void()
  ret i64 %r18
}

define i64 @lum_contar_primos(i64 %p0) {
entry:
  %cell_lim = alloca [80 x i8]
  %r0 = call i64 @_lw_void()
  %r1 = ptrtoint [80 x i8]* %cell_lim to i64
  call void @_lw_store_slot_direct(i64 %r1, i64 %r0)
  %r2 = ptrtoint [80 x i8]* %cell_lim to i64
  call void @_lw_store_slot_direct(i64 %r2, i64 %p0)
  %var_c = alloca i64
  store i64 %r0, i64* %var_c
  %var_k = alloca i64
  store i64 %r0, i64* %var_k
  %r3 = call i64 @_lw_int(i64 0)
  %r4 = call i64 @_lw_dcp(i64 %r3)
  store i64 %r4, i64* %var_c
  %r5 = call i64 @_lw_int(i64 2)
  %r6 = call i64 @_lw_dcp(i64 %r5)
  store i64 %r6, i64* %var_k
  br label %L_10
L_10:
  %r8 = load i64, i64* %var_k
  %r10 = ptrtoint [80 x i8]* %cell_lim to i64
  %r11 = call i64 @_lw_load_slot(i64 %r10)
  %r12 = call i64 @_lw_bin(i64 9, i64 %r8, i64 %r11)
  %r13 = call i64 @_lw_err_active()
  %r14 = icmp ne i64 %r13, 0
  br i1 %r14, label %ec_d_15, label %ec_o_16
ec_d_15:
  %r17 = call i64 @_lw_void()
  ret i64 %r17
ec_o_16:
  %r18 = call i64 @_lw_truthy_i(i64 %r12)
  %r19 = icmp eq i64 %r18, 0
  br i1 %r19, label %L_11, label %jf_20
jf_20:
  %r22 = load i64, i64* %var_k
  %r23 = call i64 @_lw_dcp(i64 %r22)
  %r24 = call i64 @lum_es_primo(i64 %r23)
  %r25 = call i64 @_lw_err_active()
  %r26 = icmp ne i64 %r25, 0
  br i1 %r26, label %ec_d_27, label %ec_o_28
ec_d_27:
  %r29 = call i64 @_lw_void()
  ret i64 %r29
ec_o_28:
  %r30 = call i64 @_lw_truthy_i(i64 %r24)
  %r31 = icmp eq i64 %r30, 0
  br i1 %r31, label %L_12, label %jf_32
jf_32:
  %r34 = load i64, i64* %var_c
  %r35 = call i64 @_lw_int(i64 1)
  %r36 = call i64 @_lw_bin(i64 1, i64 %r34, i64 %r35)
  %r37 = call i64 @_lw_err_active()
  %r38 = icmp ne i64 %r37, 0
  br i1 %r38, label %ec_d_39, label %ec_o_40
ec_d_39:
  %r41 = call i64 @_lw_void()
  ret i64 %r41
ec_o_40:
  %r42 = call i64 @_lw_dcp(i64 %r36)
  store i64 %r42, i64* %var_c
  br label %L_13
L_12:
  br label %L_13
L_13:
  %r44 = load i64, i64* %var_k
  %r45 = call i64 @_lw_int(i64 1)
  %r46 = call i64 @_lw_bin(i64 1, i64 %r44, i64 %r45)
  %r47 = call i64 @_lw_err_active()
  %r48 = icmp ne i64 %r47, 0
  br i1 %r48, label %ec_d_49, label %ec_o_50
ec_d_49:
  %r51 = call i64 @_lw_void()
  ret i64 %r51
ec_o_50:
  %r52 = call i64 @_lw_dcp(i64 %r46)
  store i64 %r52, i64* %var_k
  br label %L_10
L_11:
  %r54 = load i64, i64* %var_c
  ret i64 %r54
}

define i64 @lum_es_primo(i64 %p0) {
entry:
  %cell_n = alloca [80 x i8]
  %r0 = call i64 @_lw_void()
  %r1 = ptrtoint [80 x i8]* %cell_n to i64
  call void @_lw_store_slot_direct(i64 %r1, i64 %r0)
  %r2 = ptrtoint [80 x i8]* %cell_n to i64
  call void @_lw_store_slot_direct(i64 %r2, i64 %p0)
  %var_i = alloca i64
  store i64 %r0, i64* %var_i
  %r4 = ptrtoint [80 x i8]* %cell_n to i64
  %r5 = call i64 @_lw_load_slot(i64 %r4)
  %r6 = call i64 @_lw_int(i64 2)
  %r7 = call i64 @_lw_bin(i64 9, i64 %r5, i64 %r6)
  %r8 = call i64 @_lw_err_active()
  %r9 = icmp ne i64 %r8, 0
  br i1 %r9, label %ec_d_10, label %ec_o_11
ec_d_10:
  %r12 = call i64 @_lw_void()
  ret i64 %r12
ec_o_11:
  %r13 = call i64 @_lw_truthy_i(i64 %r7)
  %r14 = icmp eq i64 %r13, 0
  br i1 %r14, label %L_0, label %jf_15
jf_15:
  %r16 = call i64 @_lw_bool(i64 0)
  ret i64 %r16
L_0:
  br label %L_1
L_1:
  %r18 = ptrtoint [80 x i8]* %cell_n to i64
  %r19 = call i64 @_lw_load_slot(i64 %r18)
  %r20 = call i64 @_lw_int(i64 2)
  %r21 = call i64 @_lw_bin(i64 7, i64 %r19, i64 %r20)
  %r22 = call i64 @_lw_err_active()
  %r23 = icmp ne i64 %r22, 0
  br i1 %r23, label %ec_d_24, label %ec_o_25
ec_d_24:
  %r26 = call i64 @_lw_void()
  ret i64 %r26
ec_o_25:
  %r27 = call i64 @_lw_truthy_i(i64 %r21)
  %r28 = icmp eq i64 %r27, 0
  br i1 %r28, label %L_2, label %jf_29
jf_29:
  %r30 = call i64 @_lw_bool(i64 1)
  ret i64 %r30
L_2:
  br label %L_3
L_3:
  %r32 = ptrtoint [80 x i8]* %cell_n to i64
  %r33 = call i64 @_lw_load_slot(i64 %r32)
  %r34 = call i64 @_lw_int(i64 2)
  %r35 = call i64 @_lw_bin(i64 6, i64 %r33, i64 %r34)
  %r36 = call i64 @_lw_err_active()
  %r37 = icmp ne i64 %r36, 0
  br i1 %r37, label %ec_d_38, label %ec_o_39
ec_d_38:
  %r40 = call i64 @_lw_void()
  ret i64 %r40
ec_o_39:
  %r41 = call i64 @_lw_int(i64 0)
  %r42 = call i64 @_lw_bin(i64 7, i64 %r35, i64 %r41)
  %r43 = call i64 @_lw_err_active()
  %r44 = icmp ne i64 %r43, 0
  br i1 %r44, label %ec_d_45, label %ec_o_46
ec_d_45:
  %r47 = call i64 @_lw_void()
  ret i64 %r47
ec_o_46:
  %r48 = call i64 @_lw_truthy_i(i64 %r42)
  %r49 = icmp eq i64 %r48, 0
  br i1 %r49, label %L_4, label %jf_50
jf_50:
  %r51 = call i64 @_lw_bool(i64 0)
  ret i64 %r51
L_4:
  br label %L_5
L_5:
  %r52 = call i64 @_lw_int(i64 3)
  %var_i_53 = alloca i64
  %r54 = call i64 @_lw_dcp(i64 %r52)
  store i64 %r54, i64* %var_i_53
  br label %L_6
L_6:
  %r56 = load i64, i64* %var_i_53
  %r58 = load i64, i64* %var_i_53
  %r59 = call i64 @_lw_bin(i64 4, i64 %r56, i64 %r58)
  %r60 = call i64 @_lw_err_active()
  %r61 = icmp ne i64 %r60, 0
  br i1 %r61, label %ec_d_62, label %ec_o_63
ec_d_62:
  %r64 = call i64 @_lw_void()
  ret i64 %r64
ec_o_63:
  %r66 = ptrtoint [80 x i8]* %cell_n to i64
  %r67 = call i64 @_lw_load_slot(i64 %r66)
  %r68 = call i64 @_lw_bin(i64 10, i64 %r59, i64 %r67)
  %r69 = call i64 @_lw_err_active()
  %r70 = icmp ne i64 %r69, 0
  br i1 %r70, label %ec_d_71, label %ec_o_72
ec_d_71:
  %r73 = call i64 @_lw_void()
  ret i64 %r73
ec_o_72:
  %r74 = call i64 @_lw_truthy_i(i64 %r68)
  %r75 = icmp eq i64 %r74, 0
  br i1 %r75, label %L_7, label %jf_76
jf_76:
  %r78 = ptrtoint [80 x i8]* %cell_n to i64
  %r79 = call i64 @_lw_load_slot(i64 %r78)
  %r81 = load i64, i64* %var_i_53
  %r82 = call i64 @_lw_bin(i64 6, i64 %r79, i64 %r81)
  %r83 = call i64 @_lw_err_active()
  %r84 = icmp ne i64 %r83, 0
  br i1 %r84, label %ec_d_85, label %ec_o_86
ec_d_85:
  %r87 = call i64 @_lw_void()
  ret i64 %r87
ec_o_86:
  %r88 = call i64 @_lw_int(i64 0)
  %r89 = call i64 @_lw_bin(i64 7, i64 %r82, i64 %r88)
  %r90 = call i64 @_lw_err_active()
  %r91 = icmp ne i64 %r90, 0
  br i1 %r91, label %ec_d_92, label %ec_o_93
ec_d_92:
  %r94 = call i64 @_lw_void()
  ret i64 %r94
ec_o_93:
  %r95 = call i64 @_lw_truthy_i(i64 %r89)
  %r96 = icmp eq i64 %r95, 0
  br i1 %r96, label %L_8, label %jf_97
jf_97:
  %r98 = call i64 @_lw_bool(i64 0)
  ret i64 %r98
L_8:
  br label %L_9
L_9:
  %r100 = load i64, i64* %var_i_53
  %r101 = call i64 @_lw_int(i64 2)
  %r102 = call i64 @_lw_bin(i64 1, i64 %r100, i64 %r101)
  %r103 = call i64 @_lw_err_active()
  %r104 = icmp ne i64 %r103, 0
  br i1 %r104, label %ec_d_105, label %ec_o_106
ec_d_105:
  %r107 = call i64 @_lw_void()
  ret i64 %r107
ec_o_106:
  %r108 = call i64 @_lw_dcp(i64 %r102)
  store i64 %r108, i64* %var_i_53
  br label %L_6
L_7:
  %r109 = call i64 @_lw_bool(i64 1)
  ret i64 %r109
}

define i32 @main() {
entry:
  %res = call i64 @lum___main__()
  ret i32 0
}
