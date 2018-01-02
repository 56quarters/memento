#include <stdio.h>
#include "include/memento.h"

int main(int argc, char **argv) {
    MementoResult res = memento_fetch_path("something/foo.wsp", 100, 200);
    mement_result_free(&res);
    return 0;
}
