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
declare i64 @_lw_arr_len_i(i64)
declare i64 @_lw_to_text_i(i64)
declare i64 @_lw_concat3(i64, i64, i64)
declare i64 @_lw_concat3_i(i64, i64, i64)
declare i64 @_lw_concat3_len_i(i64, i64, i64)

@lw_str_0 = private unnamed_addr constant [8 x i8] c"arrays:\00"

define i64 @lum___main__() {
entry:
  %r0 = call i64 @_lw_void()
  %r1 = getelementptr [8 x i8], [8 x i8]* @lw_str_0, i64 0, i64 0
  %r2 = ptrtoint i8* %r1 to i64
  %r3 = call i64 @_lw_str(i64 %r2)
  %r4 = call i64 @_lw_int(i64 200000)
  %r5 = call i64 @_lw_dcp(i64 %r4)
  %r6 = call i64 @lum_bench_arrays(i64 %r5)
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

define i64 @lum_bench_arrays(i64 %p0) {
entry:
  %cell_n = alloca [80 x i8]
  %r0 = call i64 @_lw_void()
  %r1 = ptrtoint [80 x i8]* %cell_n to i64
  call void @_lw_store_slot_direct(i64 %r1, i64 %r0)
  %r2 = ptrtoint [80 x i8]* %cell_n to i64
  call void @_lw_store_slot_direct(i64 %r2, i64 %p0)
  %var_xs = alloca i64
  store i64 %r0, i64* %var_xs
  %var_i = alloca i64
  store i64 %r0, i64* %var_i
  %var_acc = alloca i64
  store i64 %r0, i64* %var_acc
  %var_j = alloca i64
  store i64 %r0, i64* %var_j
  %mg_0_0 = alloca i64
  %mg_0_1 = alloca i64
  %r3 = call i64 @_lw_arr_new()
  %r4 = call i64 @_lw_dcp(i64 %r3)
  store i64 %r4, i64* %var_xs
  %r5 = call i64 @_lw_int(i64 0)
  %r6 = call i64 @_lw_dcp(i64 %r5)
  store i64 %r6, i64* %var_i
  store i64 %r0, i64* %mg_0_0
  store i64 %r0, i64* %mg_0_1
  br label %L_0
L_0:
  %r7 = load i64, i64* %mg_0_0
  %r8 = load i64, i64* %mg_0_1
  %r10 = load i64, i64* %var_i
  %r12 = ptrtoint [80 x i8]* %cell_n to i64
  %r13 = call i64 @_lw_load_slot(i64 %r12)
  %r14 = call i64 @_lw_bin(i64 9, i64 %r10, i64 %r13)
  %r15 = call i64 @_lw_err_active()
  %r16 = icmp ne i64 %r15, 0
  br i1 %r16, label %ec_d_17, label %ec_o_18
ec_d_17:
  %r19 = call i64 @_lw_void()
  ret i64 %r19
ec_o_18:
  %r20 = call i64 @_lw_truthy_i(i64 %r14)
  %r21 = icmp eq i64 %r20, 0
  br i1 %r21, label %L_1, label %jf_22
jf_22:
  %r24 = load i64, i64* %var_xs
  %r26 = load i64, i64* %var_i
  %r28 = load i64, i64* %var_xs
  %r27 = call i64 @_lw_arr_push_ip(i64 %r28, i64 %r26)
  store i64 %r27, i64* %var_xs
  %r30 = load i64, i64* %var_i
  %r31 = call i64 @_lw_int(i64 1)
  %r32 = call i64 @_lw_bin(i64 1, i64 %r30, i64 %r31)
  %r33 = call i64 @_lw_err_active()
  %r34 = icmp ne i64 %r33, 0
  br i1 %r34, label %ec_d_35, label %ec_o_36
ec_d_35:
  %r37 = call i64 @_lw_void()
  ret i64 %r37
ec_o_36:
  %r38 = call i64 @_lw_dcp(i64 %r32)
  store i64 %r38, i64* %var_i
  store i64 %r24, i64* %mg_0_0
  store i64 %r27, i64* %mg_0_1
  br label %L_0
L_1:
  %r39 = call i64 @_lw_int(i64 0)
  %r40 = call i64 @_lw_dcp(i64 %r39)
  store i64 %r40, i64* %var_acc
  %r41 = call i64 @_lw_int(i64 0)
  %r42 = call i64 @_lw_dcp(i64 %r41)
  store i64 %r42, i64* %var_j
  br label %L_2
L_2:
  %r44 = load i64, i64* %var_j
  %r46 = ptrtoint [80 x i8]* %cell_n to i64
  %r47 = call i64 @_lw_load_slot(i64 %r46)
  %r48 = call i64 @_lw_bin(i64 9, i64 %r44, i64 %r47)
  %r49 = call i64 @_lw_err_active()
  %r50 = icmp ne i64 %r49, 0
  br i1 %r50, label %ec_d_51, label %ec_o_52
ec_d_51:
  %r53 = call i64 @_lw_void()
  ret i64 %r53
ec_o_52:
  %r54 = call i64 @_lw_truthy_i(i64 %r48)
  %r55 = icmp eq i64 %r54, 0
  br i1 %r55, label %L_3, label %jf_56
jf_56:
  %r58 = load i64, i64* %var_acc
  %r60 = load i64, i64* %var_xs
  %r62 = load i64, i64* %var_j
  %r63 = call i64 @_lw_arr_get(i64 %r60, i64 %r62)
  %r64 = call i64 @_lw_err_active()
  %r65 = icmp ne i64 %r64, 0
  br i1 %r65, label %ec_d_66, label %ec_o_67
ec_d_66:
  %r68 = call i64 @_lw_void()
  ret i64 %r68
ec_o_67:
  %r69 = call i64 @_lw_bin(i64 1, i64 %r58, i64 %r63)
  %r70 = call i64 @_lw_err_active()
  %r71 = icmp ne i64 %r70, 0
  br i1 %r71, label %ec_d_72, label %ec_o_73
ec_d_72:
  %r74 = call i64 @_lw_void()
  ret i64 %r74
ec_o_73:
  %r75 = call i64 @_lw_dcp(i64 %r69)
  store i64 %r75, i64* %var_acc
  %r77 = load i64, i64* %var_j
  %r78 = call i64 @_lw_int(i64 1)
  %r79 = call i64 @_lw_bin(i64 1, i64 %r77, i64 %r78)
  %r80 = call i64 @_lw_err_active()
  %r81 = icmp ne i64 %r80, 0
  br i1 %r81, label %ec_d_82, label %ec_o_83
ec_d_82:
  %r84 = call i64 @_lw_void()
  ret i64 %r84
ec_o_83:
  %r85 = call i64 @_lw_dcp(i64 %r79)
  store i64 %r85, i64* %var_j
  br label %L_2
L_3:
  %r87 = load i64, i64* %var_acc
  ret i64 %r87
}

define i32 @main() {
entry:
  %res = call i64 @lum___main__()
  ret i32 0
}
