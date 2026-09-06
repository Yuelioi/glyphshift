#pragma once
#ifndef GLYPHSHIFT_SYNTHETIC_FIXTURE

static HRESULT prepare_fallback_font_helpers(IMetaDataEmit* emit, mdTypeDef bridge_type, mdTypeRef string_ref, mdModuleRef native_module) {
    IMetaDataImport* metadata = nullptr;
    CHECK(emit->QueryInterface(IID_IMetaDataImport, reinterpret_cast<void**>(&metadata)));
    mdToken scope = 0; wchar_t name[256]; ULONG used = 0;
    HRESULT result = metadata->GetTypeRefProps(string_ref, &scope, name, 256, &used);
    mdTypeDef font = 0; mdMethodDef characters = 0;
    if (SUCCEEDED(result)) result = metadata->FindTypeDefByName(L"Microsoft.Xna.Framework.Graphics.SpriteFont", mdTokenNil, &font);
    if (SUCCEEDED(result)) result = metadata->FindMethod(font, L"get_Characters", nullptr, 0, &characters);
    mdTypeDef owner; ULONG signature_size = 0, rva; DWORD attributes, implementation; PCCOR_SIGNATURE getter = nullptr;
    if (SUCCEEDED(result)) result = metadata->GetMethodProps(characters, &owner, nullptr, 0, &used, &attributes, &getter, &signature_size, &rva, &implementation);
    mdTypeSpec collection = 0;
    if (SUCCEEDED(result) && signature_size > 2) result = emit->GetTokenFromTypeSpec(getter + 2, signature_size - 2, &collection);
    metadata->Release();
    if (FAILED(result)) return result;

    mdTypeRef object_type, byte_type, bool_type, assembly_type, type_type, method_type, method_base, exception_type, builder_type;
    CHECK(emit->DefineTypeRefByName(scope, L"System.Object", &object_type));
    CHECK(emit->DefineTypeRefByName(scope, L"System.Byte", &byte_type));
    CHECK(emit->DefineTypeRefByName(scope, L"System.Boolean", &bool_type));
    CHECK(emit->DefineTypeRefByName(scope, L"System.Reflection.Assembly", &assembly_type));
    CHECK(emit->DefineTypeRefByName(scope, L"System.Type", &type_type));
    CHECK(emit->DefineTypeRefByName(scope, L"System.Reflection.MethodInfo", &method_type));
    CHECK(emit->DefineTypeRefByName(scope, L"System.Reflection.MethodBase", &method_base));
    CHECK(emit->DefineTypeRefByName(scope, L"System.Exception", &exception_type));
    CHECK(emit->DefineTypeRefByName(scope, L"System.Text.StringBuilder", &builder_type));
    mdMemberRef load, get_type, get_method, invoke, contains, character_at, to_string, builder_ctor;
    std::vector<BYTE> signature{IMAGE_CEE_CS_CALLCONV_DEFAULT, 1};
    append_type(signature, ELEMENT_TYPE_CLASS, assembly_type); signature.insert(signature.end(), {ELEMENT_TYPE_SZARRAY, ELEMENT_TYPE_U1});
    CHECK(emit->DefineMemberRef(assembly_type, L"Load", signature.data(), static_cast<ULONG>(signature.size()), &load));
    signature = {IMAGE_CEE_CS_CALLCONV_HASTHIS, 1}; append_type(signature, ELEMENT_TYPE_CLASS, type_type); signature.push_back(ELEMENT_TYPE_STRING);
    CHECK(emit->DefineMemberRef(assembly_type, L"GetType", signature.data(), static_cast<ULONG>(signature.size()), &get_type));
    signature = {IMAGE_CEE_CS_CALLCONV_HASTHIS, 1}; append_type(signature, ELEMENT_TYPE_CLASS, method_type); signature.push_back(ELEMENT_TYPE_STRING);
    CHECK(emit->DefineMemberRef(type_type, L"GetMethod", signature.data(), static_cast<ULONG>(signature.size()), &get_method));
    const BYTE invoke_signature[] = {IMAGE_CEE_CS_CALLCONV_HASTHIS, 2, ELEMENT_TYPE_OBJECT, ELEMENT_TYPE_OBJECT, ELEMENT_TYPE_SZARRAY, ELEMENT_TYPE_OBJECT};
    CHECK(emit->DefineMemberRef(method_base, L"Invoke", invoke_signature, sizeof(invoke_signature), &invoke));
    const BYTE contains_signature[] = {IMAGE_CEE_CS_CALLCONV_HASTHIS, 1, ELEMENT_TYPE_BOOLEAN, ELEMENT_TYPE_VAR, 0};
    CHECK(emit->DefineMemberRef(collection, L"Contains", contains_signature, sizeof(contains_signature), &contains));
    const BYTE char_signature[] = {IMAGE_CEE_CS_CALLCONV_HASTHIS, 1, ELEMENT_TYPE_CHAR, ELEMENT_TYPE_I4};
    CHECK(emit->DefineMemberRef(string_ref, L"get_Chars", char_signature, sizeof(char_signature), &character_at));
    const BYTE string_signature[] = {IMAGE_CEE_CS_CALLCONV_HASTHIS, 0, ELEMENT_TYPE_STRING};
    CHECK(emit->DefineMemberRef(builder_type, L"ToString", string_signature, sizeof(string_signature), &to_string));
    const BYTE constructor_signature[] = {IMAGE_CEE_CS_CALLCONV_HASTHIS, 1, ELEMENT_TYPE_VOID, ELEMENT_TYPE_STRING};
    CHECK(emit->DefineMemberRef(builder_type, L".ctor", constructor_signature, sizeof(constructor_signature), &builder_ctor));

    mdFieldDef cached_method;
    signature = {IMAGE_CEE_CS_CALLCONV_FIELD}; append_type(signature, ELEMENT_TYPE_CLASS, method_type);
    CHECK(emit->DefineField(bridge_type, L"FallbackMethod", fdPrivate | fdStatic, signature.data(), static_cast<ULONG>(signature.size()), ELEMENT_TYPE_VOID, nullptr, 0, &cached_method));
    mdMethodDef read_byte, initializer;
    const BYTE byte_signature[] = {IMAGE_CEE_CS_CALLCONV_DEFAULT, 1, ELEMENT_TYPE_U1, ELEMENT_TYPE_I4};
    CHECK(emit->DefineMethod(bridge_type, L"ReadFallbackByte", mdPrivate | mdStatic | mdPinvokeImpl, byte_signature, sizeof(byte_signature), 0, miPreserveSig, &read_byte));
    CHECK(emit->DefinePinvokeMap(read_byte, pmCallConvCdecl | pmNoMangle, L"glyphshift_fallback_byte", native_module));
    const BYTE init_signature[] = {IMAGE_CEE_CS_CALLCONV_DEFAULT, 0, ELEMENT_TYPE_VOID};
    CHECK(emit->DefineMethod(bridge_type, L".cctor", mdPrivate | mdStatic | mdSpecialName | mdRTSpecialName, init_signature, sizeof(init_signature), 0, miIL, &initializer));
    mdString helper_name, method_name;
    const wchar_t type_name[] = L"Glyphshift.MonoGame.FontFallback";
    CHECK(emit->DefineUserString(type_name, static_cast<ULONG>(wcslen(type_name)), &helper_name));
    CHECK(emit->DefineUserString(L"Resolve", 7, &method_name));
    enum { loop = 1, next, original, success, handler, end };
    IlBuilder init;
    init.op(0x20); uint32_t length = sizeof(fallback_assembly);
    auto length_bytes = reinterpret_cast<const BYTE*>(&length); init.code.insert(init.code.end(), length_bytes, length_bytes + 4);
    init.call(byte_type, 0x8d); init.op(0x0a); init.op(0x16); init.op(0x0b);
    init.label(loop); init.op(0x07); init.op(0x06); init.op(0x8e); init.op(0x69); init.branch(0x3c, next);
    init.op(0x06); init.op(0x07); init.op(0x07); init.call(read_byte); init.op(0x9c);
    init.op(0x07); init.op(0x17); init.op(0x58); init.op(0x0b); init.branch(0x38, loop);
    init.label(next); init.op(0x06); init.call(load); init.call(helper_name, 0x72); init.call(get_type, 0x6f);
    init.call(method_name, 0x72); init.call(get_method, 0x6f); init.call(cached_method, 0x80); init.branch(0xdd, end);
    init.label(handler); init.op(0x26); init.branch(0xdd, end); init.label(end); init.op(0x2a);
    CHECK(set_helper_body(emit, initializer, init, {IMAGE_CEE_CS_CALLCONV_LOCAL_SIG, 2, ELEMENT_TYPE_SZARRAY, ELEMENT_TYPE_U1, ELEMENT_TYPE_I4}, exception_type, handler, end));

    const BYTE resolver_signature[] = {IMAGE_CEE_CS_CALLCONV_DEFAULT, 3, ELEMENT_TYPE_OBJECT, ELEMENT_TYPE_OBJECT, ELEMENT_TYPE_STRING, ELEMENT_TYPE_BOOLEAN};
    CHECK(emit->DefineMethod(bridge_type, L"ResolveFallback", mdPublic | mdStatic, resolver_signature, sizeof(resolver_signature), 0, miIL, &fallback_resolver));
    IlBuilder resolver;
    resolver.op(0x02); resolver.op(0x0a);
    resolver.call(cached_method, 0x7e); resolver.branch(0x39, original);
    resolver.call(cached_method, 0x7e); resolver.op(0x14); resolver.op(0x19); resolver.call(object_type, 0x8d);
    for (BYTE i = 0; i < 3; i++) {
        resolver.op(0x25); resolver.op(static_cast<BYTE>(0x16 + i)); resolver.op(static_cast<BYTE>(0x02 + i));
        if (i == 2) resolver.call(bool_type, 0x8c);
        resolver.op(0xa2);
    }
    resolver.call(invoke, 0x6f); resolver.op(0x0a);
    resolver.label(original); resolver.branch(0xdd, end);
    resolver.label(handler); resolver.op(0x26); resolver.branch(0xdd, end);
    resolver.label(end); resolver.op(0x06); resolver.op(0x2a);
    CHECK(set_helper_body(emit, fallback_resolver, resolver, {IMAGE_CEE_CS_CALLCONV_LOCAL_SIG, 1, ELEMENT_TYPE_OBJECT}, exception_type, handler, end));

    signature = {IMAGE_CEE_CS_CALLCONV_DEFAULT, 3, ELEMENT_TYPE_VOID, ELEMENT_TYPE_BYREF};
    append_type(signature, ELEMENT_TYPE_CLASS, font); signature.insert(signature.end(), {ELEMENT_TYPE_BYREF, ELEMENT_TYPE_STRING, ELEMENT_TYPE_BOOLEAN});
    CHECK(emit->DefineMethod(bridge_type, L"PrepareFontText", mdPublic | mdStatic, signature.data(), static_cast<ULONG>(signature.size()), 0, miIL, &font_bridge));
    IlBuilder text;
    text.op(0x02); text.op(0x50); text.branch(0x39, original);
    text.op(0x03); text.op(0x50); text.branch(0x39, original);
    text.op(0x03); text.op(0x50); text.op(0x25); text.call(string_length, 0x6f); text.call(bridge_method); text.op(0x0a);
    text.op(0x06); text.branch(0x39, original);
    text.op(0x02); text.op(0x50); text.op(0x06); text.op(0x04); text.call(fallback_resolver); text.call(font, 0x74); text.op(0x0b);
    text.op(0x07); text.branch(0x39, original);
    text.op(0x16); text.op(0x0c);
    text.label(loop); text.op(0x08); text.op(0x06); text.call(string_length, 0x6f); text.branch(0x3c, success);
    for (BYTE newline : {BYTE{10}, BYTE{13}}) {
        text.op(0x06); text.op(0x08); text.call(character_at, 0x6f); text.op(0x1f); text.op(newline); text.branch(0x3b, next);
    }
    text.op(0x07); text.call(characters, 0x6f); text.op(0x06); text.op(0x08); text.call(character_at, 0x6f); text.call(contains, 0x6f); text.branch(0x39, original);
    text.label(next); text.op(0x08); text.op(0x17); text.op(0x58); text.op(0x0c); text.branch(0x38, loop);
    text.label(success);
    text.op(0x19); text.call(object_type, 0x8d);
    text.op(0x25); text.op(0x16); text.op(0x02); text.op(0x50); text.op(0xa2);
    text.op(0x25); text.op(0x17); text.op(0x07); text.op(0xa2);
    text.op(0x25); text.op(0x18); text.op(0x03); text.op(0x50); text.op(0xa2);
    text.op(0x06); text.op(0x04); text.call(fallback_resolver); text.call(string_ref, 0x75);
    text.op(0x25); text.branch(0x3a, 20); text.op(0x26); text.op(0x06); text.label(20); text.op(0x0a);
    text.op(0x02); text.op(0x07); text.op(0x51); text.op(0x03); text.op(0x06); text.op(0x51);
    text.label(original); text.op(0x2a);
    std::vector<BYTE> locals{IMAGE_CEE_CS_CALLCONV_LOCAL_SIG, 3, ELEMENT_TYPE_STRING}; append_type(locals, ELEMENT_TYPE_CLASS, font); locals.push_back(ELEMENT_TYPE_I4);
    CHECK(set_helper_body(emit, font_bridge, text, locals));

    signature = {IMAGE_CEE_CS_CALLCONV_DEFAULT, 3, ELEMENT_TYPE_VOID, ELEMENT_TYPE_BYREF}; append_type(signature, ELEMENT_TYPE_CLASS, font);
    signature.push_back(ELEMENT_TYPE_BYREF); append_type(signature, ELEMENT_TYPE_CLASS, builder_type); signature.push_back(ELEMENT_TYPE_BOOLEAN);
    CHECK(emit->DefineMethod(bridge_type, L"PrepareBuilder", mdPublic | mdStatic, signature.data(), static_cast<ULONG>(signature.size()), 0, miIL, &builder_bridge));
    IlBuilder builder;
    builder.op(0x03); builder.op(0x50); builder.branch(0x39, original);
    builder.op(0x03); builder.op(0x50); builder.call(to_string, 0x6f); builder.op(0x25); builder.op(0x0a); builder.op(0x0b);
    builder.op(0x02); builder.op(0x12); builder.op(0); builder.op(0x04); builder.call(font_bridge);
    builder.op(0x06); builder.op(0x07); builder.branch(0x3b, original);
    builder.op(0x03); builder.op(0x06); builder.call(builder_ctor, 0x73); builder.op(0x51);
    builder.label(original); builder.op(0x2a);
    return set_helper_body(emit, builder_bridge, builder, {IMAGE_CEE_CS_CALLCONV_LOCAL_SIG, 2, ELEMENT_TYPE_STRING, ELEMENT_TYPE_STRING});
}
#endif
