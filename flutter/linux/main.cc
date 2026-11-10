#include <dlfcn.h>
#include "my_application.h"

#define RUNEDESK_LIB_PATH "librunedesk.so"
// #define RUNEDESK_LIB_PATH "/usr/lib/runedesk/librunedesk.so"
typedef bool (*RuneDeskCoreMain)();
bool gIsConnectionManager = false;

bool flutter_runedesk_core_main() {
   void* librunedesk = dlopen(RUNEDESK_LIB_PATH, RTLD_LAZY);
   if (!librunedesk) {
     fprintf(stderr,"load librunedesk.so failed\n");
     return true;
   }
   auto core_main = (RuneDeskCoreMain) dlsym(librunedesk,"runedesk_core_main");
   char* error;
   if ((error = dlerror()) != nullptr) {
       fprintf(stderr, "error finding runedesk_core_main: %s", error);
       return true;
   }
   return core_main();
}

int main(int argc, char** argv) {
  if (!flutter_runedesk_core_main()) {
      return 0;
  }
  for (int i = 0; i < argc; i++) {
    if (strcmp(argv[i], "--cm") == 0) {
      gIsConnectionManager = true;
    }
  }
  g_autoptr(MyApplication) app = my_application_new();
  return g_application_run(G_APPLICATION(app), argc, argv);
}
