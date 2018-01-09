#include <stdio.h>
#include <time.h>
#include "memento.h"

static void print_results(MementoResult *res) {
    for (int i = 0; i < res->size; i++) {
        printf("%u: %f\n", res->points[i].timestamp, res->points[i].value);
    }

}

int main(int argc, char **argv) {
    time_t now = time(NULL);
    MementoResult *res = memento_result_fetch("../tests/count_01.wsp", 100, now);

    if (memento_result_is_error(res)) {
        fprintf(stderr, "Failure getting results!\n");
        memento_result_free(res);
        return 1;
    }

    print_results(res);
    memento_result_free(res);

    return 0;
}
