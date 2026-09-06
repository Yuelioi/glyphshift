#pragma once
#include <map>

// Strict MonoGame signatures; no application-specific methods are selected.
static std::wstring read_type(IMetaDataImport* metadata, const BYTE*& cursor, const BYTE* end) {
    if (cursor >= end) return L"?";
    BYTE kind = *cursor++;
    switch (kind) {
    case ELEMENT_TYPE_VOID: return L"void";
    case ELEMENT_TYPE_STRING: return L"string";
    case ELEMENT_TYPE_R4: return L"float";
    case ELEMENT_TYPE_CLASS:
    case ELEMENT_TYPE_VALUETYPE: {
        ULONG encoded = 0, length = 0;
        if (FAILED(CorSigUncompressData(cursor, static_cast<DWORD>(end - cursor), &encoded, &length))) return L"?";
        cursor += length;
        mdToken token = (encoded >> 2) | ((encoded & 3) == 0 ? mdtTypeDef : ((encoded & 3) == 1 ? mdtTypeRef : mdtTypeSpec));
        wchar_t name[256]; ULONG used = 0; HRESULT result = E_FAIL;
        if (TypeFromToken(token) == mdtTypeDef) {
            DWORD flags; mdToken parent;
            result = metadata->GetTypeDefProps(token, name, 256, &used, &flags, &parent);
        } else if (TypeFromToken(token) == mdtTypeRef) {
            mdToken scope;
            result = metadata->GetTypeRefProps(token, &scope, name, 256, &used);
        }
        return SUCCEEDED(result) && used <= 256 ? name : L"?";
    }
    default: return L"?";
    }
}

static HRESULT find_monogame_targets(IMetaDataImport* metadata) {
    for (const auto type_name : {L"Microsoft.Xna.Framework.Graphics.SpriteBatch", L"Microsoft.Xna.Framework.Graphics.SpriteFont"}) {
        mdTypeDef type;
        CHECK(metadata->FindTypeDefByName(type_name, mdTokenNil, &type));
        bool draw = wcscmp(type_name, L"Microsoft.Xna.Framework.Graphics.SpriteBatch") == 0;
        HCORENUM enumeration = nullptr;
        mdMethodDef method; ULONG fetched;
        while (metadata->EnumMethods(&enumeration, type, &method, 1, &fetched) == S_OK && fetched) {
            mdTypeDef owner; wchar_t name[256]; ULONG used, signature_size, rva; DWORD flags, implementation;
            PCCOR_SIGNATURE signature;
            if (FAILED(metadata->GetMethodProps(method, &owner, name, 256, &used, &flags, &signature, &signature_size, &rva, &implementation))) continue;
            if (wcscmp(name, draw ? L"DrawString" : L"MeasureString") || signature_size < 3 || signature[0] != IMAGE_CEE_CS_CALLCONV_HASTHIS) continue;
            const BYTE* cursor = signature + 2;
            const BYTE* end = signature + signature_size;
            auto result = read_type(metadata, cursor, end);
            std::vector<std::wstring> parameters;
            for (BYTE i = 0; i < signature[1]; i++) parameters.push_back(read_type(metadata, cursor, end));
            if (cursor != end || (flags & mdStatic)) continue;
            bool builder = false, accepted = false;
            if (draw && result == L"void" && (parameters.size() == 4 || parameters.size() == 9)) {
                builder = parameters[1] == L"System.Text.StringBuilder";
                accepted = parameters[0] == L"Microsoft.Xna.Framework.Graphics.SpriteFont"
                    && (parameters[1] == L"string" || builder)
                    && parameters[2] == L"Microsoft.Xna.Framework.Vector2" && parameters[3] == L"Microsoft.Xna.Framework.Color";
                if (parameters.size() == 9) accepted = accepted && parameters[4] == L"float"
                    && parameters[5] == L"Microsoft.Xna.Framework.Vector2" && parameters[6] == L"Microsoft.Xna.Framework.Vector2"
                    && parameters[7] == L"Microsoft.Xna.Framework.Graphics.SpriteEffects" && parameters[8] == L"float";
                // Scalar-scale overloads forward to vector-scale leaves; patch only leaves.
            } else if (!draw && result == L"Microsoft.Xna.Framework.Vector2" && parameters.size() == 1) {
                builder = parameters[0] == L"System.Text.StringBuilder";
                accepted = parameters[0] == L"string" || builder;
            }
            if (accepted) targets.push_back({method, {}, static_cast<BYTE>(draw ? 2 : 1), draw ? 1 : 0, builder});
        }
        metadata->CloseEnum(enumeration);
    }
    if (targets.size() != 6) return E_NOTIMPL;
#ifndef GLYPHSHIFT_SYNTHETIC_FIXTURE
    mdTypeDef device = 0; mdMethodDef present = 0;
    CHECK(metadata->FindTypeDefByName(L"Microsoft.Xna.Framework.Graphics.GraphicsDevice", mdTokenNil, &device));
    const BYTE signature[] = {IMAGE_CEE_CS_CALLCONV_HASTHIS, 0, ELEMENT_TYPE_VOID};
    CHECK(metadata->FindMethod(device, L"Present", signature, sizeof(signature), &present));
    targets.push_back({present, {}, 0, -1, false, true});
#endif
    return S_OK;
}

struct IlBuilder {
    std::vector<BYTE> code;
    std::map<int, size_t> labels;
    std::vector<std::pair<size_t, int>> branches;
    void op(BYTE value) { code.push_back(value); }
    void call(mdToken token, BYTE opcode = 0x28) {
        op(opcode);
        auto bytes = reinterpret_cast<const BYTE*>(&token);
        code.insert(code.end(), bytes, bytes + 4);
    }
    void label(int label) { labels[label] = code.size(); }
    void branch(BYTE opcode, int label) { op(opcode); branches.push_back({code.size(), label}); code.resize(code.size() + 4); }
    void finish() {
        for (const auto& [offset, label] : branches) {
            int32_t relative = static_cast<int32_t>(labels.at(label)) - static_cast<int32_t>(offset + 4);
            memcpy(code.data() + offset, &relative, 4);
        }
    }
};

static void append_type(std::vector<BYTE>& signature, BYTE kind, mdToken type) {
    signature.push_back(kind);
    BYTE compressed[4];
    ULONG size = CorSigCompressToken(type, compressed);
    signature.insert(signature.end(), compressed, compressed + size);
}

static HRESULT set_helper_body(IMetaDataEmit* emit, mdMethodDef method, IlBuilder& il, const std::vector<BYTE>& local_signature,
    mdTypeRef exception = 0, int handler_label = 0, int end_label = 0) {
    il.finish();
    mdSignature locals = 0;
    if (!local_signature.empty()) CHECK(emit->GetTokenFromSig(local_signature.data(), static_cast<ULONG>(local_signature.size()), &locals));
    std::vector<BYTE> body(12);
    uint16_t flags = exception ? 0x301b : 0x3013, max_stack = 8;
    uint32_t size = static_cast<uint32_t>(il.code.size());
    memcpy(body.data(), &flags, 2); memcpy(body.data() + 2, &max_stack, 2);
    memcpy(body.data() + 4, &size, 4); memcpy(body.data() + 8, &locals, 4);
    body.insert(body.end(), il.code.begin(), il.code.end());
    if (exception) {
        while (body.size() % 4) body.push_back(0);
        body.insert(body.end(), {0x41, 28, 0, 0});
        uint32_t handler = static_cast<uint32_t>(il.labels.at(handler_label));
        uint32_t end = static_cast<uint32_t>(il.labels.at(end_label));
        for (uint32_t value : {0u, 0u, handler, handler, end - handler, static_cast<uint32_t>(exception)}) {
            const auto bytes = reinterpret_cast<const BYTE*>(&value); body.insert(body.end(), bytes, bytes + 4);
        }
    }
    IMethodMalloc* allocator = nullptr;
    CHECK(info->GetILFunctionBodyAllocator(target_module, &allocator));
    auto allocated = static_cast<BYTE*>(allocator->Alloc(static_cast<ULONG>(body.size())));
    allocator->Release();
    if (!allocated) return E_OUTOFMEMORY;
    memcpy(allocated, body.data(), body.size());
    return info->SetILFunctionBody(target_module, method, allocated);
}

#include "font_fallback_il.h"

static HRESULT prepare_font_helpers(IMetaDataEmit* emit, mdTypeDef bridge_type, mdTypeRef string_ref, mdModuleRef native_module) {
#ifndef GLYPHSHIFT_SYNTHETIC_FIXTURE
    return prepare_fallback_font_helpers(emit, bridge_type, string_ref, native_module);
#else
    (void)native_module;
    IMetaDataImport* metadata = nullptr;
    CHECK(emit->QueryInterface(IID_IMetaDataImport, reinterpret_cast<void**>(&metadata)));
    mdTypeDef font = 0; mdMethodDef characters = 0;
    HRESULT result = metadata->FindTypeDefByName(L"Microsoft.Xna.Framework.Graphics.SpriteFont", mdTokenNil, &font);
    if (SUCCEEDED(result)) result = metadata->FindMethod(font, L"get_Characters", nullptr, 0, &characters);
    mdTypeDef owner; ULONG name_length = 0, signature_size = 0, rva; DWORD flags, implementation; PCCOR_SIGNATURE getter_signature = nullptr;
    if (SUCCEEDED(result)) result = metadata->GetMethodProps(characters, &owner, nullptr, 0, &name_length, &flags, &getter_signature, &signature_size, &rva, &implementation);
    mdTypeSpec collection = 0;
    if (SUCCEEDED(result) && signature_size > 2) result = emit->GetTokenFromTypeSpec(getter_signature + 2, signature_size - 2, &collection);
    else result = E_FAIL;
    mdTypeRef builder = 0; HCORENUM types = nullptr; mdTypeRef candidate; ULONG fetched;
    while (metadata->EnumTypeRefs(&types, &candidate, 1, &fetched) == S_OK && fetched) {
        wchar_t name[256]; ULONG used; mdToken scope;
        if (SUCCEEDED(metadata->GetTypeRefProps(candidate, &scope, name, 256, &used)) && !wcscmp(name, L"System.Text.StringBuilder")) builder = candidate;
    }
    metadata->CloseEnum(types); metadata->Release();
    if (FAILED(result) || !builder) return FAILED(result) ? result : E_FAIL;

    mdMemberRef contains = 0, character_at = 0, builder_to_string = 0, builder_ctor = 0;
    const BYTE contains_sig[] = {IMAGE_CEE_CS_CALLCONV_HASTHIS, 1, ELEMENT_TYPE_BOOLEAN, ELEMENT_TYPE_VAR, 0};
    CHECK(emit->DefineMemberRef(collection, L"Contains", contains_sig, sizeof(contains_sig), &contains));
    const BYTE char_sig[] = {IMAGE_CEE_CS_CALLCONV_HASTHIS, 1, ELEMENT_TYPE_CHAR, ELEMENT_TYPE_I4};
    CHECK(emit->DefineMemberRef(string_ref, L"get_Chars", char_sig, sizeof(char_sig), &character_at));
    const BYTE to_string_sig[] = {IMAGE_CEE_CS_CALLCONV_HASTHIS, 0, ELEMENT_TYPE_STRING};
    CHECK(emit->DefineMemberRef(builder, L"ToString", to_string_sig, sizeof(to_string_sig), &builder_to_string));
    const BYTE ctor_sig[] = {IMAGE_CEE_CS_CALLCONV_HASTHIS, 1, ELEMENT_TYPE_VOID, ELEMENT_TYPE_STRING};
    CHECK(emit->DefineMemberRef(builder, L".ctor", ctor_sig, sizeof(ctor_sig), &builder_ctor));
    std::vector<BYTE> signature{IMAGE_CEE_CS_CALLCONV_DEFAULT, 2, ELEMENT_TYPE_STRING};
    append_type(signature, ELEMENT_TYPE_CLASS, font); signature.push_back(ELEMENT_TYPE_STRING);
    CHECK(emit->DefineMethod(bridge_type, L"TranslateFontText", mdPublic | mdStatic, signature.data(), static_cast<ULONG>(signature.size()), 0, miIL, &font_bridge));

    IlBuilder text;
    enum { original = 1, loop, next, success };
    text.op(0x02); text.branch(0x39, original); // null font preserves the original exception path
    text.op(0x03); text.branch(0x39, original);
    text.op(0x03); text.op(0x03); text.call(string_length, 0x6f); text.call(bridge_method); text.op(0x0a);
    text.op(0x06); text.branch(0x39, original);
    text.op(0x16); text.op(0x0b);
    text.label(loop);
    text.op(0x07); text.op(0x06); text.call(string_length, 0x6f); text.branch(0x3c, success);
    for (BYTE newline : {BYTE{10}, BYTE{13}}) {
        text.op(0x06); text.op(0x07); text.call(character_at, 0x6f); text.op(0x1f); text.op(newline); text.branch(0x3b, next);
    }
    text.op(0x02); text.call(characters, 0x6f);
    text.op(0x06); text.op(0x07); text.call(character_at, 0x6f); text.call(contains, 0x6f); text.branch(0x39, original);
    text.label(next); text.op(0x07); text.op(0x17); text.op(0x58); text.op(0x0b); text.branch(0x38, loop);
    text.label(success); text.op(0x06); text.op(0x2a);
    text.label(original); text.op(0x03); text.op(0x2a);
    CHECK(set_helper_body(emit, font_bridge, text, {IMAGE_CEE_CS_CALLCONV_LOCAL_SIG, 2, ELEMENT_TYPE_STRING, ELEMENT_TYPE_I4}));

    signature = {IMAGE_CEE_CS_CALLCONV_DEFAULT, 2};
    append_type(signature, ELEMENT_TYPE_CLASS, builder);
    append_type(signature, ELEMENT_TYPE_CLASS, font); append_type(signature, ELEMENT_TYPE_CLASS, builder);
    CHECK(emit->DefineMethod(bridge_type, L"TranslateBuilder", mdPublic | mdStatic, signature.data(), static_cast<ULONG>(signature.size()), 0, miIL, &builder_bridge));
    IlBuilder buffer;
    buffer.op(0x03); buffer.branch(0x39, original);
    buffer.op(0x03); buffer.call(builder_to_string, 0x6f); buffer.op(0x0a);
    buffer.op(0x02); buffer.op(0x06); buffer.call(font_bridge); buffer.op(0x0b);
    buffer.op(0x06); buffer.op(0x07); buffer.branch(0x3b, original);
    buffer.op(0x07); buffer.call(builder_ctor, 0x73); buffer.op(0x2a);
    buffer.label(original); buffer.op(0x03); buffer.op(0x2a);
    return set_helper_body(emit, builder_bridge, buffer, {IMAGE_CEE_CS_CALLCONV_LOCAL_SIG, 2, ELEMENT_TYPE_STRING, ELEMENT_TYPE_STRING});
#endif
}
