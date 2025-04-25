; ModuleID = 'autocfg_887c73f3f133753d_4.8ee5f87e536e1c2f-cgu.0'
source_filename = "autocfg_887c73f3f133753d_4.8ee5f87e536e1c2f-cgu.0"
target datalayout = "e-m:o-p270:32:32-p271:32:32-p272:64:64-i64:64-i128:128-f80:128-n8:16:32:64-S128"
target triple = "x86_64-apple-macosx10.12.0"

@alloc_e6758488a51c40069ade2309416f0500 = private unnamed_addr constant <{ [6 x i8] }> <{ [6 x i8] c"<anon>" }>, align 1
@alloc_4258b12ada5ef08aac481ecd3933272a = private unnamed_addr constant <{ ptr, [16 x i8] }> <{ ptr @alloc_e6758488a51c40069ade2309416f0500, [16 x i8] c"\06\00\00\00\00\00\00\00\01\00\00\00\1F\00\00\00" }>, align 8

; autocfg_887c73f3f133753d_4::probe
; Function Attrs: uwtable
define void @_ZN26autocfg_887c73f3f133753d_45probe17h3b9a9b4240d609daE() unnamed_addr #0 {
start:
  ret void
}

; core::panicking::panic_const::panic_const_div_by_zero
; Function Attrs: cold noinline noreturn uwtable
declare void @_ZN4core9panicking11panic_const23panic_const_div_by_zero17h6400326194e5be11E(ptr align 8) unnamed_addr #1

attributes #0 = { uwtable "frame-pointer"="all" "probe-stack"="inline-asm" "target-cpu"="penryn" }
attributes #1 = { cold noinline noreturn uwtable "frame-pointer"="all" "probe-stack"="inline-asm" "target-cpu"="penryn" }

!llvm.module.flags = !{!0}
!llvm.ident = !{!1}

!0 = !{i32 8, !"PIC Level", i32 2}
!1 = !{!"rustc version 1.85.1 (4eb161250 2025-03-15)"}
