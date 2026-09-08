// Independent lifecycle model of the documented VGUI lookup -> SetText copy.
// This executable does not load, hook, or discover a real framework library.
#include <deque>
#include <iostream>
#include <stdexcept>
#include <string>
#include <utility>

static void require(bool condition, const char* message) {
    if (!condition) throw std::runtime_error(message);
}

struct Table {
    const std::wstring original = L"Open";
    unsigned findIndex(const char* token) const {
        return std::string(token) == "menu.open" ? 0u : ~0u;
    }
    const wchar_t* value(unsigned index) const {
        return index == 0 ? original.c_str() : nullptr;
    }
};

// A stand-in for an adapter's query transformation. Buffers remain alive even
// when publications change, because query callers may retain returned pointers.
struct QueryBridge {
    const Table& table;
    bool enabled = false;
    bool interceptNames = true;
    bool interceptIndices = true;
    std::wstring translation;
    std::deque<std::wstring> retainedResults;

    const wchar_t* transform(const wchar_t* source, bool intercepted) {
        if (!source || !enabled || !intercepted || translation.empty()) return source;
        if (source != table.original) return source;
        retainedResults.push_back(translation);
        return retainedResults.back().c_str();
    }
    const wchar_t* find(const char* token) {
        return transform(table.value(table.findIndex(token)), interceptNames);
    }
    const wchar_t* byIndex(unsigned index) {
        return transform(table.value(index), interceptIndices);
    }
};

struct Control {
    std::wstring copiedText;
    void setText(const wchar_t* text) { copiedText = text ? text : L""; }
    const std::wstring& paint() const { return copiedText; }
};

int main() {
    try {
        Table table;
        QueryBridge bridge{table};
        bridge.translation = L"\u6253\u5f00";
        Control existing;
        existing.setText(bridge.byIndex(0));
        bridge.enabled = true;
        require(existing.paint() == L"Open", "late activation cannot refresh a copied control");
        std::cout << "PASS late_activation_preserves_existing_copy\n";

        bridge.interceptIndices = false;
        Control named, indexed;
        named.setText(bridge.find("menu.open"));
        indexed.setText(bridge.byIndex(0));
        require(named.paint() == bridge.translation && indexed.paint() == L"Open",
            "Find-only interception must expose the uncovered index path");
        std::cout << "PASS find_only_does_not_cover_index_lookup\n";

        bridge.interceptIndices = true;
        indexed.setText(bridge.byIndex(0));
        require(indexed.paint() == bridge.translation, "both lookup paths must translate");
        const wchar_t* retained = bridge.byIndex(0);
        bridge.translation = L"\u7b2c\u4e8c\u4ee3";
        require(indexed.paint() == L"\u6253\u5f00", "publication alone cannot change the copied control");
        indexed.setText(bridge.byIndex(0));
        require(indexed.paint() == bridge.translation, "reconstruction must use the new publication");
        require(std::wstring(retained) == L"\u6253\u5f00", "retained query pointers must survive updates");
        std::cout << "PASS generation_change_requires_new_set_text\n";

        bridge.enabled = false;
        require(std::wstring(bridge.byIndex(0)) == L"Open", "disabled queries must return original text");
        require(indexed.paint() == bridge.translation, "query restoration is not control restoration");
        indexed.setText(bridge.byIndex(0));
        require(indexed.paint() == L"Open", "reconstruction after disable must restore original text");
        require(std::wstring(retained) == L"\u6253\u5f00", "disable must not free retained query buffers");
        std::cout << "PASS query_and_control_restoration_are_separate\n";

        bridge.enabled = true;
        require(!bridge.find("unknown") && !bridge.byIndex(~0u), "missing tokens must remain missing");
        require(table.original == L"Open", "the original localization table must remain unchanged");
        std::cout << "PASS missing_tokens_and_original_table_preserved\n";

        Control firstCreatedAfterActivation;
        firstCreatedAfterActivation.setText(bridge.byIndex(0));
        require(firstCreatedAfterActivation.paint() == bridge.translation,
            "activation before initial lookup must cover a newly created control");
        std::cout << "PASS_activation_before_control_creation\n";
        std::cout << "6 lifecycle cases passed; no real-software compatibility claimed\n";
        return 0;
    } catch (const std::exception& error) {
        std::cerr << "FAIL " << error.what() << '\n';
        return 1;
    }
}
