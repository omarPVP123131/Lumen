/* LUMEN AOT — compiled to C */
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <math.h>

typedef struct { int tag; int64_t i; double f; char* s; int b; void* ptr; } Val;
#define TAG_INT 0
#define TAG_FLOAT 1
#define TAG_STR 2
#define TAG_BOOL 3

static Val g_vars[256];
static const char* g_var_names[256];
static int g_var_count = 0;
static int _find_var(const char* n) { for(int i=0;i<g_var_count;i++) if(!strcmp(g_var_names[i],n)) return i; g_var_names[g_var_count]=n; return g_var_count++; } 
static char* _fmt(Val v) { char* b=malloc(128); if(v.tag==TAG_INT) snprintf(b,128,"%lld",(long long)v.i); else if(v.tag==TAG_FLOAT) snprintf(b,128,"%g",v.f); else if(v.tag==TAG_STR) snprintf(b,128,"%s",v.s?v.s:""); else if(v.tag==TAG_BOOL) snprintf(b,128,"%s",v.b?"true":"false"); else b[0]=0; return b; }
static char* _strcat(char* a, char* b) { char* r=malloc(strlen(a)+strlen(b)+1); strcpy(r,a); strcat(r,b); return r; }
static char* _itoa(int64_t n) { char* b=malloc(32); snprintf(b,32,"%lld",(long long)n); return b; }
static char* _ftoa(double n) { char* b=malloc(64); snprintf(b,64,"%g",n); return b; }

typedef struct { Val* data; int len; int cap; } Arr;
static Arr* _arr_new() { Arr* a=malloc(sizeof(Arr)); a->cap=8; a->data=malloc(8*sizeof(Val)); a->len=0; return a; }
static void _arr_push(Arr* a, Val v) { if(a->len>=a->cap){a->cap*=2;a->data=realloc(a->data,a->cap*sizeof(Val));} a->data[a->len++]=v; }
static Val _arr_get(Arr* a, int i) { return (i<0||i>=a->len)?(Val){0}:a->data[i]; }
static void _arr_set(Arr* a, int i, Val v) { if(i>=0&&i<a->len) a->data[i]=v; }

void _f___main__();

void _f___main__() {
  Val s0={TAG_STR,.s=(char*)"¡Hola, LÚMEN!"};
  /* call imprimir with 1 args */
  Val s1={TAG_INT,.i=0};
  return;
}

int main() { _f___main__(); return 0; }
