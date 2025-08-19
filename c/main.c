#define JRSL_IMPLEMENTATION
#include "jrsl.h"

/* The probability to add a new level to the skip list*/
#define P 0.5f

/* A helper function used during the skip list's destruction */
void free_data(void *key, void *data) { free(data); }

void label_printer(void *key, void *data) {
  // Print the key:data format and capture how many characters were printed
  int printed_chars = printf("%d:%c", *(int*)key, *(char*)data);
  
  // Pad with spaces to make total width 6 characters
  printf("%*s", 6 - printed_chars, "");
}

int intcmp(const void *a, const void *b) {
    int ia = *(const int*)a;
    int ib = *(const int*)b;
    return (ia > ib) - (ia < ib);
}

int main() {

  skip_list_t skip_list;

  int keys[] =  {3,  6,  7,  12, 19, 17, 26, 21, 25, 21};
  char data[] = {'b','g','g','m','l','u','l','t','w','c'};

  size_t i; /* used in for loops */

  /* Let's initialize the skip list using jrsl_initialize, we eill use a
   * probability of 0.5. We can use jrsl_max_level to determine the optimum
   * amount of levels knowing the maximum size of our list.*/

  printf("max level: %d", jrsl_max_level(10, P));
  jrsl_initialize(&skip_list, (comparator_t)intcmp, free, P,
                  jrsl_max_level(10, P));

  printf("\n\nEmpty skip list\n");
  jrsl_display_list(&skip_list, label_printer);

  /* Inserting elements into the skip list to fill it up. We won't be using any
   * useful data, we will just malloc for every letter. */
  printf("\n\nInserting elements\n");
  for (i = 0; i < 10; i++) {
    jrsl_insert(&skip_list, &keys[i], &data[i]);
  }
  jrsl_display_list(&skip_list, label_printer);
  printf("\n");
  return 0;
}
