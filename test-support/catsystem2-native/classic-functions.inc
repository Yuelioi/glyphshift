// Synthetic register-ABI owner; no game bytes or offsets.
extern "C" __declspec(naked) void deleting() {
    __asm { push edi }
    __asm { mov edi, ecx }
    __asm { call destroy_text }
    __asm { pop edi }
    __asm { ret 4 }
}
extern "C" __declspec(naked) void destroy_text() {
#ifdef WRONG_MEMBER
    __asm { mov eax, [edi + TextMember + 4] }
#else
    __asm { mov eax, [edi + TextMember] }
#endif
    __asm { mov dword ptr [edi], offset object_table + 4 }
    __asm { push eax }
    __asm { call free }
    __asm { add esp, 4 }
    __asm { mov dword ptr [edi + TextMember], 0 }
    __asm { ret }
}
extern "C" __declspec(naked) void set_text() {
    __asm {
        push ebx
        push esi
        mov eax, [edi + TextMember]
        push eax
        call free
        add esp, 4
        push 32768
        call malloc
        add esp, 4
        mov [edi + TextMember], eax
        mov ebx, eax
        test esi, esi
        jz empty_text
    copy_text:
        mov cl, [esi]
        mov [ebx], cl
        inc esi
        inc ebx
        test cl, cl
        jne copy_text
        jmp copied
    empty_text:
        mov byte ptr [ebx], 0
    copied:
        mov dword ptr [edi + GlyphMember], -1
        pop esi
        pop ebx
        mov eax, 1
        ret
    }
}
extern "C" __declspec(naked) void finish_text() {
    __asm { mov dword ptr [eax + FinishMember], 1 }
    __asm { ret }
}
extern "C" __declspec(naked) void show_text() {
    __asm { mov [eax + LimitMember], ecx }
    __asm { mov ecx, 1 }
    __asm { mov [eax + DirtyMember], ecx }
#ifdef WRONG_DISPLAY
    __asm { mov dword ptr [eax + GlyphMember], 2 }
#else
    __asm { mov [eax + GlyphMember], ecx }
#endif
    __asm { ret }
}
extern "C" __declspec(naked) void append_text() {
    __asm {
        push esi
        push edi
        mov esi, [esp + 12]
        mov eax, [ebx + TextMember]
        mov [ebx + TextMember], eax
        mov edi, eax
    find_end:
        cmp byte ptr [edi], 0
        je found
        inc edi
        jmp find_end
    found:
        test esi, esi
        jz done
    copy_loop:
        mov cl, [esi]
        mov [edi], cl
        inc esi
        inc edi
        test cl, cl
        jne copy_loop
    done:
        mov dword ptr [ebx + GlyphMember], -1
        pop edi
        pop esi
        ret 4
    }
}
extern "C" const unsigned char end_marker[] = {255, 0};
extern "C" __declspec(naked) void set_command() {
    __asm { push ebp }
    __asm { mov ebp, esp }
    __asm { sub esp, 16 }
    __asm { push edi }
    __asm { push esi }
    __asm { mov edi, [ebp + 8] }
    __asm { mov byte ptr [ebp - 16], 0 }
    __asm { lea esi, [ebp - 16] }
    __asm { call set_text }
    __asm { pop esi }
    __asm { pop edi }
    __asm { mov esp, ebp }
    __asm { pop ebp }
    __asm { ret 16 }
}
extern "C" __declspec(naked) void append_command() {
    __asm { push ebp }
    __asm { mov ebp, esp }
    __asm { sub esp, 16 }
    __asm { push ebx }
    __asm { mov eax, [ebp + 8] }
    __asm { mov byte ptr [ebp - 16], 0 }
    __asm { lea edx, [ebp - 16] }
    __asm { push edx }
    __asm { mov ebx, eax }
    __asm { call append_text }
    __asm { pop ebx }
    __asm { mov esp, ebp }
    __asm { pop ebp }
    __asm { ret 16 }
}
extern "C" __declspec(naked) void marker_command() {
    __asm { push ebx }
    __asm { mov ebx, [esp + 8] }
    __asm { push offset end_marker }
    __asm { call append_text }
    __asm { pop ebx }
    __asm { ret 16 }
}
extern "C" __declspec(naked) void skip_command() {
    __asm { mov eax, [esp + 4] }
    __asm { mov dword ptr [eax + FinishMember], 1 }
    __asm { ret 16 }
}
