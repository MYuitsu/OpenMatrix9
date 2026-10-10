#pragma once
#include <cstddef>
namespace App {class DocumentObject;}
namespace OpenMatrix9Gui {
class CoreNotes {
public:
    static void registerTypes();
    static void activate();
    static void deactivate();
    static bool handles(std::size_t command);
    static bool available(std::size_t command);
    static bool execute(std::size_t command);
    static const char* alias(std::size_t command);
    static bool isStorageObject(const App::DocumentObject* object);
};
}
