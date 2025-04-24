; ModuleID = 'autocfg_887c73f3f133753d_3.d152d22dc997a64a-cgu.0'
source_filename = "autocfg_887c73f3f133753d_3.d152d22dc997a64a-cgu.0"
target datalayout = "e-m:o-p270:32:32-p271:32:32-p272:64:64-i64:64-i128:128-f80:128-n8:16:32:64-S128"
target triple = "x86_64-apple-macosx10.12.0"

; autocfg_887c73f3f133753d_3::probe
; Function Attrs: uwtable
define void @_ZN26autocfg_887c73f3f133753d_35probe17h07d3811416871132E() unnamed_addr #0 {
start:
  %0 = alloca [4 x i8], align 4
  store i32 1, ptr %0, align 4
  %_0.i = load i32, ptr %0, align 4
  ret void
}

; Function Attrs: nocallback nofree nosync nounwind speculatable willreturn memory(none)
declare i32 @llvm.cttz.i32(i32, i1 immarg) #1

attributes #0 = { uwtable "frame-pointer"="all" "probe-stack"="inline-asm" "target-cpu"="penryn" }
attributes #1 = { nocallback nofree nosync nounwind speculatable willreturn memory(none) }

!llvm.module.flags = !{!0}
!llvm.ident = !{!1}

!0 = !{i32 8, !"PIC Level", i32 2}
!1 = !{!"rustc version 1.85.1 (4eb161250 2025-03-15)"}
