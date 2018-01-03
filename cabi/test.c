#include <stdio.h>
#include <time.h>
#include "include/memento.h"

int main(int argc, char **argv) {
    time_t now = time(NULL);
    MementoResult res = memento_fetch_path("../tests/count_01.wsp", 100, now);
    mement_result_free(&res);
    return 0;
}
