#ifndef LUMEN_RT_H
#define LUMEN_RT_H

#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <math.h>
#include <time.h>

/* POSIX headers for non-Windows (Linux + macOS) */
#if !defined(_WIN32)
#include <unistd.h>
#include <pthread.h>
#if !defined(__APPLE__)
#include <regex.h>
#include <dirent.h>
#endif
#endif

#ifdef _WIN32
#include <windows.h>
#include <process.h>
#else
extern char** environ;
#endif

typedef struct Val {
  int t;
  int64_t i;
  double f;
  const char* s;
  int argc;
  struct Val* items;
  struct Val (*fp)(void);
  const char* en;
  const char* vr;
  struct Val* p;
} Val;

#define T_INT 0
#define T_FLT 1
#define T_STR 2
#define T_BOL 3
#define T_ARR 4
#define T_TUP 5
#define T_VOD 6
#define T_OK  7
#define T_ERR 8
#define T_SOM 9
#define T_NON 10
#define T_ENM 11
#define T_FRE 12
#define T_STT 13
#define T_MAP 14
#define T_PTR 15

static int64_t _rt_pid(void) {
#ifdef _WIN32
  return (int64_t)GetCurrentProcessId();
#else
  return (int64_t)getpid();
#endif
}

static Val ST[16384];
static int SP = 0;
#define PUSH(v) (ST[(SP)++] = (v))
/* Guard contra underflow: las funciones void retornan sin valor apilado;
   el VM hace pop().unwrap_or(Void), aquí devolvemos Void igualmente. */
#define POP() ((SP) > 0 ? ST[--(SP)] : _v_void())
#define TOP() (ST[SP - 1])

static Val gv[16384];
static const char* gn[16384];
static int gc = 0;

/* ── intentar/atrapar (sin unwinding: bandera de error + chequeos estáticos) ──
   Cada operación riesgosa pone _err=1; el código generado chequea después de
   cada una y salta al manejador abierto más cercano (etiqueta estática).
   Es determinista, seguro con -O3 y no depende de setjmp/longjmp. */
static char* _last_err_msg = 0;
static int _err = 0;
static int _h_sp[64];
static int _hn = 0;
static char* _fmt(Val v);
static void _rt_throw_val(Val v) {
  if (_hn > 0 || _err) {
    _last_err_msg = _fmt(v);
    _err = 1;
    return;
  }
  /* Sin manejador en ninguna función del camino: fallar como siempre */
  fprintf(stderr, "%s\n", _fmt(v));
  exit(3);
}
static void _rt_throw(const char* msg) {
  char* m = (char*)malloc(strlen(msg) + 1);
  strcpy(m, msg);
  _rt_throw_val((Val){.t = T_STR, .s = m});
}
/* Chequeo con manejador abierto: restaurar stack, apilar mensaje, saltar */
#define _ERRCHK(lbl)                                                          \
  do {                                                                        \
    if (__builtin_expect(_err, 0)) {                                          \
      _hn--;                                                                  \
      SP = _h_sp[_hn];                                                        \
      PUSH(_v_str(_last_err_msg));                                            \
      _err = 0;                                                               \
      goto lbl;                                                               \
    }                                                                         \
  } while (0)
/* Chequeo sin manejador en esta función: propagar al llamador */
#define _ERRPROP()                                       \
  do {                                                   \
    if (__builtin_expect(_err, 0)) return _v_void();     \
  } while (0)
static int _try_begin(void) {
  if (_hn >= 64) _rt_throw("Demasiados manejadores intentar anidados");
  _h_sp[_hn] = SP;
  return _hn++;
}
static void _try_end(void) { if (_hn > 0) _hn--; }

static void _reg(const char* n) {
  for (int i = 0; i < gc; i++)
    if (!strcmp(gn[i], n)) return;
  gn[gc] = n;
  gc++;
}

static int _fv(const char* n) {
  for (int i = 0; i < gc; i++)
    if (!strcmp(gn[i], n)) return i;
  return 0;
}

#define MAX_PARF 256
#define MAX_PARP 32
static const char* _pars[MAX_PARF][MAX_PARP];
static int _parc[MAX_PARF];
static int _parn = 0;

static void _regpars(const char* fn, const char** ps, int argc) {
  _pars[_parn][0] = fn;
  for (int i = 0; i < argc && i < MAX_PARP - 1; i++) _pars[_parn][i + 1] = ps[i];
  _parc[_parn] = argc;
  _parn++;
}

static const char* _par(const char* fn, int ix) {
  for (int k = 0; k < _parn; k++)
    if (!strcmp(_pars[k][0], fn)) return _pars[k][ix + 1];
  return "";
}

static int _parcnt(const char* fn) {
  for (int k = 0; k < _parn; k++)
    if (!strcmp(_pars[k][0], fn)) return _parc[k];
  return 0;
}

static inline Val _v_int(int64_t x) { return (Val){.i = x, .t = T_INT}; }
static inline Val _v_flt(double x) { return (Val){.f = x, .t = T_FLT}; }
static inline Val _v_bool(int x) { return (Val){.i = x ? 1 : 0, .t = T_BOL}; }
static inline Val _v_void(void) { return (Val){.t = T_VOD}; }
/* Referencia mutable (prestado mut): apunta a un slot gv[] estable. */
static inline Val _v_ptr(Val* p) { return (Val){.p = p, .t = T_PTR}; }
static inline Val _deref(Val v) {
  int guard = 0;
  while (v.t == T_PTR && v.p && guard++ < 1024) v = *v.p;
  return v;
}
static Val _v_str(const char* s) {
  size_t n = strlen(s);
  char* m = (char*)malloc(n + 1);
  memcpy(m, s, n + 1);
  Val v = _v_int(0);
  v.t = T_STR;
  v.s = m;
  return v;
}
static Val _vfref(const char* n, Val (*fp)(void)) {
  Val v = _v_int(0);
  v.t = T_FRE;
  v.s = n;
  v.fp = fp;
  return v;
}
static Val _fref_call(Val v) {
  if (!v.fp) return _v_void();
  return v.fp();
}

static int _isnum(Val v) { return v.t == T_INT || v.t == T_FLT || v.t == T_BOL; }
static double _asf(Val v) { return v.t == T_FLT ? v.f : (double)v.i; }

static inline int _eq(Val a, Val b) {
  a = _deref(a);
  b = _deref(b);
  if (__builtin_expect(a.t == T_INT && b.t == T_INT, 1)) return a.i == b.i;
  if (_isnum(a) && _isnum(b)) return _asf(a) == _asf(b);
  if (a.t == T_STR && b.t == T_STR) return strcmp(a.s, b.s) == 0;
  if (a.t != b.t) return 0;
  switch (a.t) {
    case T_ARR:
    case T_TUP:
      if (a.argc != b.argc) return 0;
      for (int i = 0; i < a.argc; i++)
        if (!_eq(a.items[i], b.items[i])) return 0;
      return 1;
    case T_SOM:
    case T_OK:
      return _eq(a.items[0], b.items[0]);
    case T_ENM:
      if (strcmp(a.en, b.en) != 0 || strcmp(a.vr, b.vr) != 0 || a.argc != b.argc) return 0;
      for (int i = 0; i < a.argc; i++)
        if (!_eq(a.items[i], b.items[i])) return 0;
      return 1;
    default:
      return a.i == b.i;
  }
}

static inline int _lts(Val a, Val b) {
  if (__builtin_expect(a.t == T_INT && b.t == T_INT, 1)) return a.i < b.i;
  if (_isnum(a) && _isnum(b)) return _asf(a) < _asf(b);
  if (a.t == T_STR && b.t == T_STR) return strcmp(a.s, b.s) < 0;
  return a.t < b.t;
}

static int _truthy(Val v) {
  v = _deref(v);
  switch (v.t) {
    case T_BOL:
    case T_INT:
      return v.i != 0;
    case T_FLT:
      return v.f != 0.0;
    case T_STR:
      return strlen(v.s) > 0;
    case T_ARR:
    case T_TUP:
      return v.argc > 0;
    case T_MAP:
      return v.argc > 0;
    case T_NON:
    case T_VOD:
      return 0;
    default:
      return 1;
  }
}

static Val _arith(int op, Val a, Val b) {
  int isf = a.t == T_FLT || b.t == T_FLT;
  if (!isf) {
    int64_t x = a.i, y = b.i;
    switch (op) {
      case 1: return _v_int(x + y);
      case 3: return _v_int(x - y);
      case 4: return _v_int(x * y);
      case 5: if (!y) { _rt_throw("Error: Division por cero"); } return _v_int(x / y);
      case 6: if (!y) { _rt_throw("Error: Division por cero"); } return _v_int(x % y);
    }
    return _v_int(0);
  }
  double x = _asf(a), y = _asf(b);
  switch (op) {
    case 1: return _v_flt(x + y);
    case 3: return _v_flt(x - y);
    case 4: return _v_flt(x * y);
    case 5: return _v_flt(x / y);
    case 6: return _v_flt(fmod(x, y));
  }
  return _v_flt(0);
}

static char* _fmt(Val v);

static inline Val _bin(int op, Val a, Val b) {
  if (__builtin_expect(a.t == T_INT && b.t == T_INT, 1)) {
    int64_t x = a.i, y = b.i;
    switch (op) {
      case 1:  return (Val){.i = x + y, .t = T_INT};
      case 3:  return (Val){.i = x - y, .t = T_INT};
      case 4:  return (Val){.i = x * y, .t = T_INT};
      case 5:  if (!y) { _rt_throw("Error: Division por cero"); } return (Val){.i = x / y, .t = T_INT};
      case 6:  if (!y) { _rt_throw("Error: Division por cero"); } return (Val){.i = x % y, .t = T_INT};
      case 7:  return (Val){.i = (x == y), .t = T_BOL};
      case 8:  return (Val){.i = (x != y), .t = T_BOL};
      case 9:  return (Val){.i = (x < y), .t = T_BOL};
      case 10: return (Val){.i = (x <= y), .t = T_BOL};
      case 11: return (Val){.i = (x > y), .t = T_BOL};
      case 12: return (Val){.i = (x >= y), .t = T_BOL};
      case 13: return (Val){.i = (x != 0 && y != 0), .t = T_BOL};
      case 14: return (Val){.i = (x != 0 || y != 0), .t = T_BOL};
      case 15: return (Val){.i = x | y, .t = T_INT};
      case 16: return (Val){.i = x & y, .t = T_INT};
      case 17: return (Val){.i = x << y, .t = T_INT};
      case 18: return (Val){.i = x >> y, .t = T_INT};
      case 19: return (Val){.i = x ^ y, .t = T_INT};
    }
  }
  if (op == 1 && (a.t == T_STR || b.t == T_STR)) {
    char* as = _fmt(a);
    char* bs = _fmt(b);
    size_t l1 = strlen(as), l2 = strlen(bs);
    char* m = (char*)malloc(l1 + l2 + 1);
    memcpy(m, as, l1);
    memcpy(m + l1, bs, l2);
    m[l1 + l2] = 0;
    return _v_str(m);
  }
  if (op >= 1 && op <= 6) return _arith(op, a, b);
  switch (op) {
    case 7:  return _v_bool(_eq(a, b));
    case 8:  return _v_bool(!_eq(a, b));
    case 9:  return _v_bool(_lts(a, b));
    case 10: return _v_bool(_lts(a, b) || _eq(a, b));
    case 11: return _v_bool(!(_lts(a, b) || _eq(a, b)));
    case 12: return _v_bool(!_lts(a, b));
    case 13: return _v_bool(_truthy(a) && _truthy(b));
    case 14: return _v_bool(_truthy(a) || _truthy(b));
    case 15: return _v_int((int64_t)_asf(a) | (int64_t)_asf(b));
    case 16: return _v_int((int64_t)_asf(a) & (int64_t)_asf(b));
    case 17: return _v_int((int64_t)_asf(a) << (int64_t)_asf(b));
    case 18: return _v_int((int64_t)_asf(a) >> (int64_t)_asf(b));
    case 19: return _v_int((int64_t)_asf(a) ^ (int64_t)_asf(b));
  }
  return _v_int(0);
}

static Val _neg(Val a) {
  if (a.t == T_FLT) return _v_flt(-a.f);
  return _v_int(-a.i);
}
static Val _not(Val a) { return _v_bool(!_truthy(a)); }
static Val _bnot(Val a) { return _v_int(~(int64_t)_asf(a)); }

static inline Val _dcp(Val v) {
  if (__builtin_expect(v.t <= T_BOL || v.t == T_STR || v.t == T_NON || v.t == T_VOD, 1)) return v;
  /* Una referencia se copia tal cual (el punto de escritura no cambia). */
  if (v.t == T_PTR || v.t == T_FRE) return v;
  if (v.t == T_ARR || v.t == T_TUP || v.t == T_ENM) {
    Val nv = v;
    nv.items = (Val*)malloc(sizeof(Val) * (v.argc > 0 ? v.argc : 1));
    for (int i = 0; i < v.argc; i++) nv.items[i] = _dcp(v.items[i]);
    return nv;
  }
  if (v.t == T_MAP) {
    Val nv = v;
    nv.items = (Val*)malloc(sizeof(Val) * (v.argc > 0 ? v.argc * 2 : 2));
    for (int i = 0; i < v.argc * 2; i++) nv.items[i] = _dcp(v.items[i]);
    return nv;
  }
  if (v.t == T_STT) {
    Val nv = v;
    nv.items = (Val*)malloc(sizeof(Val) * (v.argc > 0 ? v.argc * 2 : 2));
    for (int i = 0; i < v.argc * 2; i++) nv.items[i] = _dcp(v.items[i]);
    return nv;
  }
  return v;
}

static Val _arrn(Val* xs, int n) {
  Val v = _v_int(0);
  v.t = T_ARR;
  v.argc = n;
  v.items = (Val*)malloc(sizeof(Val) * (n > 0 ? n : 1));
  for (int i = 0; i < n; i++) v.items[i] = xs[i];
  return v;
}
static Val _tupn(Val* xs, int n) {
  Val v = _arrn(xs, n);
  v.t = T_TUP;
  return v;
}
static Val _arr_push(Val a, Val x) {
  Val* ns = (Val*)malloc(sizeof(Val) * (a.argc + 1));
  for (int i = 0; i < a.argc; i++) ns[i] = a.items[i];
  ns[a.argc] = x;
  a.argc++;
  a.items = ns;
  return a;
}
static Val _arr_get(Val a, int64_t ix) {
  /* Fuzzing 3.3.6: indexado de textos "abc"[1] (paridad con VM) */
  if (a.t == T_STR) {
    int64_t n = (int64_t)strlen(a.s ? a.s : "");
    if (ix < 0 || ix >= n) {
      char _eb[96];
      snprintf(_eb, sizeof _eb, "Índice %lld fuera de rango (largo: %lld)", (long long)ix, (long long)n);
      _rt_throw(_eb);
    }
    char c2[2] = {a.s[ix], 0};
    return _v_str(c2);
  }
  if (ix < 0 || ix >= a.argc) {
    char _eb[96];
    snprintf(_eb, sizeof _eb, "Indice %lld fuera de rango (largo: %d)", (long long)ix, a.argc);
    _rt_throw(_eb);
  }
  return a.items[ix];
}
static Val _arr_set(Val a, int64_t ix, Val x) {
  (void)x;
  if (a.t == T_STR) {
    char _eb[96];
    snprintf(_eb, sizeof _eb, "No se puede asignar a un índice de texto");
    _rt_throw(_eb);
  }
  if (ix < 0 || ix >= a.argc) {
    char _eb[96];
    snprintf(_eb, sizeof _eb, "Indice %lld fuera de rango (largo: %d)", (long long)ix, a.argc);
    _rt_throw(_eb);
  }
  a.items[ix] = x;
  return a;
}
static Val _arr_len(Val a) {
  /* Fuzzing 3.3.6: largo() sobre texto también (paridad con VM) */
  if (a.t == T_STR) return _v_int((int64_t)strlen(a.s ? a.s : ""));
  return _v_int(a.argc);
}
static Val _arr_rev(Val a) {
  Val* ns = (Val*)malloc(sizeof(Val) * (a.argc + 1));
  for (int i = 0; i < a.argc; i++) ns[i] = a.items[a.argc - 1 - i];
  return _arrn(ns, a.argc);
}
static int _asc_cmp(const void* pa, const void* pb) {
  Val a = *(Val*)pa, b = *(Val*)pb;
  double an = (a.t == T_FLT) ? a.f : (double)a.i;
  double bn = (b.t == T_FLT) ? b.f : (double)b.i;
  if (an < bn) return -1;
  if (an > bn) return 1;
  return 0;
}
static Val _arr_sort(Val a) {
  Val* ns = (Val*)malloc(sizeof(Val) * (a.argc + 1));
  for (int i = 0; i < a.argc; i++) ns[i] = a.items[i];
  if (a.argc > 1) qsort(ns, a.argc, sizeof(Val), _asc_cmp);
  return _arrn(ns, a.argc);
}

static Val _res(Val x, int ok) {
  Val v = _arrn(&x, 1);
  v.t = ok ? T_OK : T_ERR;
  return v;
}
static Val _unwrap(Val v) {
  if (v.t == T_OK || v.t == T_SOM) return v.items[0];
  return v;
}
static Val _some(Val x) {
  Val v = _arrn(&x, 1);
  v.t = T_SOM;
  return v;
}
static Val _none(void) { Val v = _v_int(0); v.t = T_NON; return v; }

static Val _enm(const char* en, const char* vr, int argc, Val* xs) {
  Val v = _v_int(0);
  v.t = T_ENM;
  v.en = en;
  v.vr = vr;
  v.argc = argc;
  v.items = (Val*)malloc(sizeof(Val) * (argc > 0 ? argc : 1));
  for (int i = 0; i < argc; i++) v.items[i] = xs[i];
  return v;
}
static Val _st_new(const char* nm, int n, Val* vs, const char** ns) {
  Val v = _v_int(0);
  v.t = T_STT;
  v.en = nm;
  v.argc = n;
  v.items = (Val*)malloc(sizeof(Val) * (n > 0 ? n * 2 : 2));
  for (int i = 0; i < n; i++) {
    v.items[2 * i] = _v_str(ns[i]);
    v.items[2 * i + 1] = vs[i];
  }
  return v;
}
static Val _st_get(Val s, const char* f) {
  for (int i = 0; i < s.argc; i++) {
    if (!strcmp(s.items[2 * i].s, f)) return s.items[2 * i + 1];
  }
  fprintf(stderr, "Campo '%s' no encontrado en struct\n", f);
  exit(3);
  return _v_int(0);
}
static Val _st_set(Val s, const char* f, Val x) {
  for (int i = 0; i < s.argc; i++) {
    if (!strcmp(s.items[2 * i].s, f)) {
      s.items[2 * i + 1] = x;
      return s;
    }
  }
  fprintf(stderr, "Campo '%s' no encontrado en struct\n", f);
  exit(3);
  return s;
}

static char* _fmt(Val v) {
  char* b = (char*)malloc(8192);
  if (!b) return (char*)"";
  v = _deref(v);
  switch (v.t) {
    case T_INT:
      snprintf(b, 8192, "%lld", (long long)v.i);
      break;
    case T_FLT: {
      double d = v.f;
      if (isinf(d)) { snprintf(b, 8192, "inf"); break; }
      if (d == (double)(int64_t)d && fabs(d) < 1e16) {
        snprintf(b, 8192, "%lld", (long long)d);
      } else {
        int _p; char _t[64];
        for (_p = 1; _p <= 17; _p++) {
          snprintf(_t, sizeof _t, "%.*g", _p, d);
          if (strtod(_t, NULL) == d) { snprintf(b, 8192, "%s", _t); break; }
        }
        if (_p > 17) snprintf(b, 8192, "%.17g", d);
      }
      break;
    }
    case T_BOL:
      snprintf(b, 8192, "%s", v.i ? "true" : "false");
      break;
    case T_STR: {
      const char* s = v.s ? v.s : "";
      size_t n = strlen(s);
      memcpy(b, s, n + 1);
      break;
    }
    case T_FRE:
      snprintf(b, 8192, "<funcion %s>", v.s ? v.s : "?");
      break;
    case T_VOD:
      snprintf(b, 8192, "void");
      break;
    case T_NON:
      snprintf(b, 8192, "ninguno");
      break;
    case T_OK: {
      char* x = _fmt(v.items[0]);
      snprintf(b, 8192, "exito(%s)", x);
      break;
    }
    case T_ERR: {
      char* x = _fmt(v.items[0]);
      snprintf(b, 8192, "error(%s)", x);
      break;
    }
    case T_SOM: {
      char* x = _fmt(v.items[0]);
      snprintf(b, 8192, "algun(%s)", x);
      break;
    }
    case T_ARR: {
      size_t off = 0;
      b[off++] = '[';
      for (int i = 0; i < v.argc; i++) {
        if (i > 0) { b[off++] = ','; b[off++] = ' '; }
        char* item = _fmt(v.items[i]);
        size_t n = strlen(item);
        memcpy(b + off, item, n);
        off += n;
      }
      b[off++] = ']';
      b[off] = 0;
      break;
    }
    case T_TUP: {
      size_t off = 0;
      b[off++] = '(';
      for (int i = 0; i < v.argc; i++) {
        if (i > 0) { b[off++] = ','; b[off++] = ' '; }
        char* item = _fmt(v.items[i]);
        size_t n = strlen(item);
        memcpy(b + off, item, n);
        off += n;
      }
      b[off++] = ')';
      b[off] = 0;
      break;
    }
    case T_ENM: {
      if (v.argc == 0) {
        snprintf(b, 8192, "%s::%s", v.en, v.vr);
      } else {
        size_t off = 0;
        int n = snprintf(b, 8192, "%s::%s(", v.en, v.vr);
        off = n > 0 ? (size_t)n : 0;
        for (int i = 0; i < v.argc; i++) {
          if (i > 0) { b[off++] = ','; b[off++] = ' '; }
          char* item = _fmt(v.items[i]);
          size_t n2 = strlen(item);
          memcpy(b + off, item, n2);
          off += n2;
        }
        b[off++] = ')';
        b[off] = 0;
      }
      break;
    }
    case T_STT: {
      size_t off = 0;
      b[off++] = '{';
      b[off++] = ' ';
      for (int i = 0; i < v.argc; i++) {
        if (i > 0) { b[off++] = ','; b[off++] = ' '; }
        char* f = _fmt(v.items[2 * i]);
        char* fv = _fmt(v.items[2 * i + 1]);
        size_t n1 = strlen(f), n2 = strlen(fv);
        memcpy(b + off, f, n1); off += n1;
        b[off++] = ':'; b[off++] = ' ';
        memcpy(b + off, fv, n2); off += n2;
      }
      b[off++] = ' ';
      b[off] = '}';
      b[off + 1] = 0;
      break;
    }
    case T_MAP: {
      size_t off = 0;
      b[off++] = '{';
      for (int i = 0; i < v.argc; i++) {
        if (i > 0) { b[off++] = ','; b[off++] = ' '; }
        char* k = _fmt(v.items[2 * i]);
        char* kv = _fmt(v.items[2 * i + 1]);
        size_t n1 = strlen(k), n2 = strlen(kv);
        memcpy(b + off, k, n1); off += n1;
        b[off++] = ':'; b[off++] = ' ';
        memcpy(b + off, kv, n2); off += n2;
      }
      b[off++] = ' ';
      b[off] = '}';
      b[off + 1] = 0;
      break;
    }
    default:
      b[0] = 0;
  }
  return b;
}

static Val _read_ln(void) {
  char buf[4096];
  if (!fgets(buf, sizeof(buf), stdin)) return _v_str("");
  size_t n = strlen(buf);
  while (n > 0 && (buf[n - 1] == '\n' || buf[n - 1] == '\r')) buf[--n] = 0;
  char* end = NULL;
  long long ll = strtoll(buf, &end, 10);
  if (end != buf && *end == 0) return _v_int(ll);
  double d = strtod(buf, &end);
  if (end != buf && *end == 0) return _v_flt(d);
  return _v_str(buf);
}

/* ==================== builtin natives ==================== */

static Val _call_by_name(const char* nm);

static Val _map_new(void) { Val v = _v_int(0); v.t = T_MAP; v.argc = 0; v.items = NULL; return v; }
static Val _map_set(Val m, Val k, Val x) {
  Val nv = m;
  int n = m.argc;
  Val* ni = (Val*)malloc(sizeof(Val) * ((size_t)n + 1) * 2);
  for (int i = 0; i < n * 2; i++) ni[i] = m.items[i];
  ni[2 * n] = k; ni[2 * n + 1] = x;
  nv.items = ni; nv.argc = n + 1;
  return nv;
}
static Val _map_get(Val m, Val k) {
  for (int i = 0; i < m.argc; i++) if (_eq(m.items[2 * i], k)) return m.items[2 * i + 1];
  return _v_void();
}
static Val _map_has(Val m, Val k) {
  for (int i = 0; i < m.argc; i++) if (_eq(m.items[2 * i], k)) return _v_bool(1);
  return _v_bool(0);
}
static Val _map_len(Val m) { return _v_int(m.argc); }
static Val _map_keys(Val m) {
  Val* ns = (Val*)malloc(sizeof(Val) * (m.argc + 1));
  for (int i = 0; i < m.argc; i++) ns[i] = m.items[2 * i];
  return _arrn(ns, m.argc);
}

static Val _set_new(void) { return _arrn(NULL, 0); }
static Val _set_add(Val s, Val x) {
  for (int i = 0; i < s.argc; i++) if (_eq(s.items[i], x)) return s;
  return _arr_push(s, x);
}
static Val _set_has(Val s, Val x) {
  for (int i = 0; i < s.argc; i++) if (_eq(s.items[i], x)) return _v_bool(1);
  return _v_bool(0);
}
static Val _set_union(Val a, Val b) {
  Val r = _arrn(NULL, 0);
  for (int i = 0; i < a.argc; i++) r = _set_add(r, a.items[i]);
  for (int i = 0; i < b.argc; i++) r = _set_add(r, b.items[i]);
  return r;
}
static Val _set_inter(Val a, Val b) {
  Val r = _arrn(NULL, 0);
  for (int i = 0; i < a.argc; i++) if (_set_has(b, a.items[i]).i) r = _set_add(r, a.items[i]);
  return r;
}
static Val _set_diff(Val a, Val b) {
  Val r = _arrn(NULL, 0);
  for (int i = 0; i < a.argc; i++) if (!_set_has(b, a.items[i]).i) r = _set_add(r, a.items[i]);
  return r;
}

static Val _bytes_arr(Val* xs, int n) { return _arrn(xs, n); }
static Val _push_front(Val a, Val x) {
  Val* ns = (Val*)malloc(sizeof(Val) * (a.argc + 1));
  ns[0] = x;
  for (int i = 0; i < a.argc; i++) ns[i + 1] = a.items[i];
  a.argc++;
  a.items = ns;
  return a;
}
static Val _pop_front(Val a) {
  if (a.argc == 0) return _v_void();
  Val r = a.items[0];
  a.argc--;
  a.items = a.items + 1;
  return r;
}
static Val _pop_back(Val a) {
  if (a.argc == 0) return _v_void();
  return a.items[a.argc - 1];
}

static int _hc_cmp(const void* pa, const void* pb) {
  Val a = *(Val*)pa, b = *(Val*)pb;
  double an = (a.t == T_FLT) ? a.f : (double)a.i;
  double bn = (b.t == T_FLT) ? b.f : (double)b.i;
  if (an < bn) return 1;
  if (an > bn) return -1;
  return 0;
}
static Val _heap_agregar(Val h, Val x) {
  Val nv = _arr_push(h, x);
  if (nv.argc > 1) qsort(nv.items, nv.argc, sizeof(Val), _hc_cmp);
  return nv;
}

/* ── Motor regex PROPIO por backtracking (v3.3.7) ─────────────────────
   Puerto fiel de crates/lumen-vm/src/min_regex.rs: misma sintaxis soportada
   (^ $ . \d \D \w \W \s \S [a-z] clases con negación, * + ?, grupos (),
   alternancia |), misma semántica greedy/backtracking — paridad exacta
   VM↔nativo en TODAS las plataformas (elimina POSIX regex y stubs). */
enum { R_LIT, R_ANY, R_DIG, R_NDIG, R_WRD, R_NWRD, R_SPC, R_NSPC,
       R_CLASS, R_QUANT, R_START, R_END, R_CAP, R_ALT };
typedef struct RPc RPc;
struct RPc {
  int t; char lit; int neg;
  char ccls[128]; int nccls;
  struct { char lo, hi; } crng[32]; int nrng;
  RPc *inner; int qmin, qmax;      /* qmax < 0 = ilimitado */
  RPc **seq; int nseq;             /* secuencia (raíz / captura) */
  int idx;
  RPc ***alts; int *alts_n; int nalts;
};
static RPc *_rp(int t) {
  RPc *p = (RPc*)calloc(1, sizeof(RPc));
  p->t = t; p->qmax = -1;
  return p;
}

static int _rp_groups = 0;
static RPc **_rp_seq(const char *s, int *i, int *out_n);
static RPc *_rp_alt(const char *s, int *i);

static RPc *_rp_piece(const char *s, int *i) {
  char c = s[*i];
  (*i)++;
  if (c == '^') return _rp(R_START);
  if (c == '$') return _rp(R_END);
  if (c == '.') return _rp(R_ANY);
  if (c == '\\') {
    if (!s[*i]) return NULL;
    char e = s[*i]; (*i)++;
    switch (e) {
      case 'd': return _rp(R_DIG);  case 'D': return _rp(R_NDIG);
      case 'w': return _rp(R_WRD);  case 'W': return _rp(R_NWRD);
      case 's': return _rp(R_SPC);  case 'S': return _rp(R_NSPC);
      default: { RPc *p = _rp(R_LIT); p->lit = e; return p; }
    }
  }
  if (c == '[') {
    RPc *p = _rp(R_CLASS);
    if (s[*i] == '^') { p->neg = 1; (*i)++; }
    while (s[*i] && s[*i] != ']') {
      if (s[*i+1] == '-' && s[*i+2] && s[*i+2] != ']' &&
          p->nrng < 32) {
        p->crng[p->nrng].lo = s[*i];
        p->crng[p->nrng].hi = s[*i+2];
        p->nrng++; (*i) += 3;
      } else if (p->nccls < 128) {
        p->ccls[p->nccls++] = s[*i]; (*i)++;
      } else (*i)++;
    }
    if (s[*i]) (*i)++; /* ']' */
    return p;
  }
  if (c == '(') {
    RPc *cap = _rp(R_CAP);
    cap->idx = ++_rp_groups;
    cap->seq = (RPc**)_rp_seq(s, i, &cap->nseq);
    if (s[*i] != ')') return NULL;
    (*i)++;
    return cap;
  }
  if (c == ')' || c == '|') return NULL;
  { RPc *p = _rp(R_LIT); p->lit = c; return p; }
}

/* Parsea una secuencia hasta '|' o ')' (o fin); devuelve NULL en error */
static RPc **_rp_seq(const char *s, int *i, int *out_n) {
  RPc **items = (RPc**)malloc(sizeof(RPc*) * strlen(s));
  int n = 0;
  while (s[*i] && s[*i] != '|' && s[*i] != ')') {
    RPc *p = _rp_piece(s, i);
    if (!p) { return NULL; }
    if (s[*i] == '*' || s[*i] == '+' || s[*i] == '?') {
      RPc *q = _rp(R_QUANT);
      q->inner = p;
      if (s[*i] == '*') { q->qmin = 0; q->qmax = -1; }
      else if (s[*i] == '+') { q->qmin = 1; q->qmax = -1; }
      else { q->qmin = 0; q->qmax = 1; }
      (*i)++;
      items[n++] = q;
      continue;
    }
    /* v3.4.2: acotador {m}, {m,}, {m,n}; malformado → '{' queda literal */
    if (s[*i] == '{') {
      int save = *i, mn = 0, has_m = 0, mx = -1;
      (*i)++;
      while (s[*i] >= '0' && s[*i] <= '9') { mn = mn * 10 + (s[*i] - '0'); has_m = 1; (*i)++; }
      if (has_m) {
        if (s[*i] == ',') {
          (*i)++;
          if (s[*i] >= '0' && s[*i] <= '9') {
            int v = 0;
            while (s[*i] >= '0' && s[*i] <= '9') { v = v * 10 + (s[*i] - '0'); (*i)++; }
            mx = v;
          } else mx = -1; /* {m,} = ilimitado */
        } else mx = mn;   /* {m} exacto */
        if (s[*i] == '}') {
          (*i)++;
          RPc *q = _rp(R_QUANT);
          q->inner = p; q->qmin = mn; q->qmax = mx < 0 ? -1 : mx;
          items[n++] = q;
          continue;
        }
      }
      *i = save; /* no válido → pieza normal y '{' literal después */
    }
    items[n++] = p;
  }
  *out_n = n;
  return items;
}

static RPc *_rp_alt(const char *s, int *i) {
  int cap = 4, n = 0;
  RPc ***alts = (RPc***)malloc(sizeof(RPc**) * cap);
  int *ns = (int*)malloc(sizeof(int) * cap);
  for (;;) {
    int sn = 0;
    RPc **seq = _rp_seq(s, i, &sn);
    if (!seq) return NULL;
    if (n == cap) { cap *= 2; alts = realloc(alts, sizeof(RPc**)*cap); ns = realloc(ns, sizeof(int)*cap); }
    alts[n] = seq; ns[n] = sn; n++;
    if (s[*i] == '|') { (*i)++; continue; }
    break;
  }
  if (n == 1) {
    RPc *root = _rp(R_CAP); root->idx = 0; root->seq = alts[0]; root->nseq = ns[0];
    return root;
  }
  /* Raíz con alternancia: envolver el ALT en un CAP raíz para que
     root->seq/nseq siempre sean válidos en los escaneos */
  RPc *alt = _rp(R_ALT);
  alt->alts = alts; alt->alts_n = ns; alt->nalts = n;
  RPc *wrap = _rp(R_CAP);
  wrap->idx = 0;
  wrap->seq = (RPc**)malloc(sizeof(RPc*));
  wrap->seq[0] = alt; wrap->nseq = 1;
  return wrap;
}

static int _risdig(char c){ return c>='0'&&c<='9'; }
static int _riswrd(char c){ return _risdig(c)||(c>='a'&&c<='z')||(c>='A'&&c<='Z')||c=='_'; }
static int _risspc(char c){ return c==' '||c=='\t'||c=='\n'||c=='\r'||c=='\f'||c=='\v'; }

static struct { int st, en; } _rcaps[16];
static int _rcaps_on = 0;
/* Backtracking: devuelve posición final o -1 (espejo de try_match_cap) */
static int _rtry(RPc **seq, int n, const char *cs, int cl, int ci, int pi) {
  while (pi < n) {
    RPc *p = seq[pi];
    switch (p->t) {
      case R_START: if (ci != 0) return -1; pi++; break;
      case R_END:   if (ci != cl) return -1; pi++; break;
      case R_ANY:   if (ci >= cl || cs[ci] == '\n') return -1; ci++; pi++; break;
      case R_LIT:   if (ci >= cl || cs[ci] != p->lit) return -1; ci++; pi++; break;
      case R_DIG:   if (ci >= cl || !_risdig(cs[ci])) return -1; ci++; pi++; break;
      case R_NDIG:  if (ci >= cl ||  _risdig(cs[ci])) return -1; ci++; pi++; break;
      case R_WRD:   if (ci >= cl || !_riswrd(cs[ci])) return -1; ci++; pi++; break;
      case R_NWRD:  if (ci >= cl ||  _riswrd(cs[ci])) return -1; ci++; pi++; break;
      case R_SPC:   if (ci >= cl || !_risspc(cs[ci])) return -1; ci++; pi++; break;
      case R_NSPC:  if (ci >= cl ||  _risspc(cs[ci])) return -1; ci++; pi++; break;
      case R_CLASS: {
        if (ci >= cl) return -1;
        char ch = cs[ci];
        int m = 0;
        for (int k = 0; k < p->nccls; k++) if (p->ccls[k] == ch) m = 1;
        for (int k = 0; k < p->nrng; k++)
          if (ch >= p->crng[k].lo && ch <= p->crng[k].hi) m = 1;
        if (p->neg) m = !m;
        if (!m) return -1;
        ci++; pi++; break;
      }
      case R_QUANT: {
        int count = 0;
        while (p->qmax < 0 || count < p->qmax) {
          int e = _rtry(&p->inner, 1, cs, cl, ci, 0);
          if (e < 0) break;
          ci = e; count++;
        }
        if (count < p->qmin) return -1;
        { int e = _rtry(seq, n, cs, cl, ci, pi + 1);
          if (e >= 0) return e; }
        return -1;
      }
      case R_CAP: {
        int e = _rtry(p->seq, p->nseq, cs, cl, ci, 0);
        if (e < 0) return -1;
        if (_rcaps_on && p->idx > 0 && p->idx < 16) { _rcaps[p->idx].st = ci; _rcaps[p->idx].en = e; }
        ci = e; pi++; break;
      }
      case R_ALT: {
        for (int k = 0; k < p->nalts; k++) {
          int e = _rtry(p->alts[k], p->alts_n[k], cs, cl, ci, 0);
          if (e >= 0) { ci = e; break; }
          if (k == p->nalts - 1) return -1;
        }
        pi++; break;
      }
      default: return -1;
    }
  }
  return ci;
}


typedef struct { RPc *root; int anchored; int ngroups; } RegexC;

static RegexC _regex_compile(const char *pat) {
  RegexC r; r.root = NULL; r.anchored = 0;
  int i = 0;
  _rp_groups = 0;
  RPc *root = _rp_alt(pat, &i);
  if (!root || pat[i]) return r;
  r.root = root;
  r.ngroups = _rp_groups;
  if (root->nseq > 0 && root->seq[0]->t == R_START) r.anchored = 1;
  return r;
}

/* ¿La raíz casa empezando en pos? Devuelve fin o -1 */
static int _rtry_root(RegexC *r, const char *cs, int cl, int pos) {
  return _rtry(r->root->seq, r->root->nseq, cs, cl, pos, 0);
}

static int _regex_m(const char* pat, const char* s);

/* Coincidencia con paridad de errores del VM: patrón malformado devuelve
   Error(texto) en vez de false silencioso (v3.4.3) */
static Val _regex_m_val(const char* pat, const char* s) {
  RegexC r = _regex_compile(pat);
  if (!r.root) {
    char msg[128];
    snprintf(msg, sizeof msg,
      "error(regex parse error:\n    %s\n     ^\nerror: unclosed counted repetition)", pat);
    Val* it = (Val*)malloc(sizeof(Val));
    it[0] = _v_str(msg);
    return (Val){ .t = T_ERR, .argc = 1, .items = it };
  }
  int cl = (int)strlen(s);
  int found = 0;
  if (r.anchored) found = _rtry_root(&r, s, cl, 0) >= 0;
  else { for (int i = 0; i <= cl && !found; i++) found = _rtry_root(&r, s, cl, i) >= 0; }
  return _v_bool(found);
}

static int _regex_m(const char* pat, const char* s) {
  RegexC r = _regex_compile(pat);
  if (!r.root) return 0;
  int cl = (int)strlen(s);
  if (r.anchored) return _rtry_root(&r, s, cl, 0) >= 0;
  for (int i = 0; i <= cl; i++)
    if (_rtry_root(&r, s, cl, i) >= 0) return 1;
  return 0;
}

/* Capturas: devuelve T_ARR de T_STR ([0]=match completo, [1..]=grupos) o vacío */
static Val _regex_caps(const char* pat, const char* s) {
  Val empty = { .t = T_ARR, .argc = 0, .items = (Val*)malloc(sizeof(Val)) };
  RegexC r = _regex_compile(pat);
  if (!r.root) return empty;
  int cl = (int)strlen(s);
  for (int pos = 0; pos <= cl; pos++) {
    memset(_rcaps, 0, sizeof(_rcaps));
    _rcaps_on = 1;
    int end = _rtry(r.root->seq, r.root->nseq, s, cl, pos, 0);
    _rcaps_on = 0;
    if (end >= 0 && r.root->idx >= 0 && r.root->idx < 16) {
      _rcaps[0].st = pos; _rcaps[0].en = end;
      int n = r.ngroups + 1;
      Val* arr = (Val*)malloc(sizeof(Val) * (n > 0 ? n : 1));
      for (int k = 0; k < n; k++) {
        int a = _rcaps[k].st, b = _rcaps[k].en;
        if (b > a && b <= cl) {
          char* m = (char*)malloc((size_t)(b - a) + 1);
          memcpy(m, s + a, (size_t)(b - a)); m[b - a] = 0;
          arr[k] = _v_str(m);
        } else arr[k] = _v_str("");
      }
      return (Val){ .t = T_ARR, .argc = n, .items = arr };
    }
  }
  return empty;
}
static char* _regex_rep(const char* pat, const char* s, const char* rep) {
  RegexC r = _regex_compile(pat);
  size_t cap = strlen(s) * 4 + strlen(rep) * 8 + 64;
  char* out = (char*)malloc(cap);
  size_t oi = 0;
  if (!r.root) { strcpy(out, s); return out; }
  int cl = (int)strlen(s);
  int pos = 0;
  while (pos < cl) {
    _rcaps_on = 1;
    memset(_rcaps, 0, sizeof(_rcaps));
    int end = _rtry_root(&r, s, cl, pos);
    if (end >= 0) {
      /* $0 = match completo: la raíz CAP no pasa por R_CAP en _rtry */
      _rcaps[0].st = pos; _rcaps[0].en = end;
      /* v3.4.0: expansión de $N y ${N} sobre las capturas */
      const char* rp = rep;
      while (*rp) {
        if (rp[0] == '$' && rp[1] == '{') {
          const char* close = strchr(rp + 2, '}');
          if (close) {
            char num[8]; size_t nl = (size_t)(close - (rp + 2));
            if (nl < sizeof num) { memcpy(num, rp + 2, nl); num[nl] = 0; }
            else num[0] = 0;
            int n = atoi(num);
            if (n >= 0 && n < 16 && r.ngroups >= n && _rcaps[n].en > _rcaps[n].st) {
              int a = _rcaps[n].st, b = _rcaps[n].en;
              while (oi + (size_t)(b - a) + 1 > cap) { cap *= 2; out = (char*)realloc(out, cap); }
              memcpy(out + oi, s + a, (size_t)(b - a)); oi += (size_t)(b - a);
            }
            rp = close + 1;
            continue;
          }
        } else if (rp[0] == '$' && rp[1] >= '0' && rp[1] <= '9') {
          int n = rp[1] - '0';
          if (n < 16 && r.ngroups >= n && _rcaps[n].en > _rcaps[n].st) {
            int a = _rcaps[n].st, b = _rcaps[n].en;
            while (oi + (size_t)(b - a) + 1 > cap) { cap *= 2; out = (char*)realloc(out, cap); }
            memcpy(out + oi, s + a, (size_t)(b - a)); oi += (size_t)(b - a);
          }
          rp += 2;
          continue;
        }
        if (oi + 2 > cap) { cap *= 2; out = (char*)realloc(out, cap); }
        out[oi++] = *rp++;
      }
      pos = end > pos ? end : pos + 1; /* evita bucle en match vacío */
      _rcaps_on = 0;
    } else {
      _rcaps_on = 0;
      if (oi + 2 > cap) { cap *= 2; out = (char*)realloc(out, cap); }
      out[oi++] = s[pos];
      pos++;
    }
  }
  out[oi] = 0;
  return out;
}

static const uint32_t _dec_tab[][3] = {
  {0x00C0,0x41,0x300},{0x00C1,0x41,0x301},{0x00C2,0x41,0x302},{0x00C3,0x41,0x303},{0x00C4,0x41,0x308},{0x00C5,0x41,0x30A},
  {0x00C7,0x43,0x327},{0x00C8,0x45,0x300},{0x00C9,0x45,0x301},{0x00CA,0x45,0x302},{0x00CB,0x45,0x308},
  {0x00CC,0x49,0x300},{0x00CD,0x49,0x301},{0x00CE,0x49,0x302},{0x00CF,0x49,0x308},
  {0x00D1,0x4E,0x303},{0x00D2,0x4F,0x300},{0x00D3,0x4F,0x301},{0x00D4,0x4F,0x302},{0x00D5,0x4F,0x303},{0x00D6,0x4F,0x308},
  {0x00D9,0x55,0x300},{0x00DA,0x55,0x301},{0x00DB,0x55,0x302},{0x00DC,0x55,0x308},
  {0x00DD,0x59,0x301},{0x00E0,0x61,0x300},{0x00E1,0x61,0x301},{0x00E2,0x61,0x302},{0x00E3,0x61,0x303},{0x00E4,0x61,0x308},{0x00E5,0x61,0x30A},
  {0x00E7,0x63,0x327},{0x00E8,0x65,0x300},{0x00E9,0x65,0x301},{0x00EA,0x65,0x302},{0x00EB,0x65,0x308},
  {0x00EC,0x69,0x300},{0x00ED,0x69,0x301},{0x00EE,0x69,0x302},{0x00EF,0x69,0x308},
  {0x00F1,0x6E,0x303},{0x00F2,0x6F,0x300},{0x00F3,0x6F,0x301},{0x00F4,0x6F,0x302},{0x00F5,0x6F,0x303},{0x00F6,0x6F,0x308},
  {0x00F9,0x75,0x300},{0x00FA,0x75,0x301},{0x00FB,0x75,0x302},{0x00FC,0x75,0x308},
  {0x00FD,0x79,0x301},{0x00FF,0x79,0x308}
};
static int _u8d(const unsigned char* s, size_t* i, uint32_t* cp) {
  unsigned char c = s[*i];
  if (c < 0x80) { *cp = c; (*i)++; return 1; }
  if ((c >> 5) == 0x6 && (s[*i + 1] & 0xC0) == 0x80 && s[*i + 1] != 0) { *cp = ((c & 0x1F) << 6) | (s[*i + 1] & 0x3F); (*i) += 2; return 1; }
  if ((c >> 4) == 0xE && (s[*i + 1] & 0xC0) == 0x80 && (s[*i + 2] & 0xC0) == 0x80 && s[*i + 1] != 0 && s[*i + 2] != 0) { *cp = ((c & 0x0F) << 12) | ((s[*i + 1] & 0x3F) << 6) | (s[*i + 2] & 0x3F); (*i) += 3; return 1; }
  *cp = c; (*i)++;
  return 1;
}
static size_t _u8e(uint32_t cp, unsigned char* b) {
  if (cp < 0x80) { b[0] = (unsigned char)cp; return 1; }
  if (cp < 0x800) { b[0] = 0xC0 | (cp >> 6); b[1] = 0x80 | (cp & 0x3F); return 2; }
  b[0] = 0xE0 | (cp >> 12); b[1] = 0x80 | ((cp >> 6) & 0x3F); b[2] = 0x80 | (cp & 0x3F); return 3;
}
static char* _norm(const char* s, const char* form) {
  uint32_t cps[256];
  size_t nc = 0, i = 0;
  while (s[i] && nc < 256) { uint32_t cp; _u8d((const unsigned char*)s, &i, &cp); cps[nc++] = cp; }
  int compose = 1;
  if (form && !strcmp(form, "NFD")) compose = 0;
  if (form && !strcmp(form, "NFKD")) compose = 0;
  uint32_t out[512];
  size_t no = 0;
  for (size_t k = 0; k < nc; k++) {
    uint32_t cp = cps[k];
    if (compose && no > 0 && cp >= 0x300 && cp <= 0x30F) {
      int found = 0;
      for (size_t t = 0; t < sizeof(_dec_tab) / sizeof(_dec_tab[0]); t++) {
        if (_dec_tab[t][1] == out[no - 1] && _dec_tab[t][2] == cp) { out[no - 1] = _dec_tab[t][0]; found = 1; break; }
      }
      if (found) continue;
    }
    if (!compose) {
      int found = 0;
      for (size_t t = 0; t < sizeof(_dec_tab) / sizeof(_dec_tab[0]); t++) {
        if (_dec_tab[t][0] == cp) { out[no++] = _dec_tab[t][1]; out[no++] = _dec_tab[t][2]; found = 1; break; }
      }
      if (found) continue;
    }
    out[no++] = cp;
  }
  unsigned char tmp[8];
  size_t total = no * 4 + 1;
  char* r = (char*)malloc(total);
  size_t ri = 0;
  for (size_t k = 0; k < no; k++) {
    size_t n = _u8e(out[k], tmp);
    memcpy(r + ri, tmp, n); ri += n;
  }
  r[ri] = 0;
  return r;
}

static char* _pad_str(const char* s, int64_t len, const char* ch, int start) {
  int64_t sl = (int64_t)strlen(s);
  int64_t need = len > sl ? len - sl : 0;
  char* m = (char*)malloc((size_t)(sl + need + 1));
  char c = ch && ch[0] ? ch[0] : ' ';
  if (start) {
    for (int64_t k = 0; k < need; k++) m[k] = c;
    memcpy(m + need, s, (size_t)sl + 1);
  } else {
    memcpy(m, s, (size_t)sl);
    for (int64_t k = 0; k < need; k++) m[sl + k] = c;
    m[sl + need] = 0;
  }
  return m;
}
static Val _utf8_bytes(const char* s) {
  size_t n = strlen(s);
  Val* xs = (Val*)malloc(sizeof(Val) * (n > 0 ? n : 1));
  for (size_t i = 0; i < n; i++) xs[i] = _v_int((unsigned char)s[i]);
  Val v = _arrn(xs, (int)n);
  free(xs);
  return v;
}
static const char* _tipo_de_b(Val v) {
  switch (v.t) {
    case T_INT: return "entero";
    case T_FLT: return "decimal";
    case T_BOL: return "booleano";
    case T_STR: return "texto";
    case T_ARR: return "lista";
    case T_MAP: return "diccionario";
    case T_VOD: return "nulo";
    case T_FRE: return "funcion";
    case T_STT: return "estructura";
    case T_ENM: return "enumeracion";
    case T_TUP: return "tupla";
    case T_OK: return "exito";
    case T_ERR: return "error";
    default: return "opcion";
  }
}
static char* _case_str(const char* s, int up) {
  size_t n = strlen(s);
  char* m = (char*)malloc(n + 1);
  for (size_t i = 0; i < n; i++) {
    char c = s[i];
    if (up && c >= 'a' && c <= 'z') c = c - 32;
    else if (!up && c >= 'A' && c <= 'Z') c = c + 32;
    m[i] = c;
  }
  m[n] = 0;
  return m;
}
static Val _time_now(void) { return _v_int((int64_t)time(NULL)); }
static int64_t _time_parse(const char* s) {
  if (!s) return 0;
  char buf[64];
  snprintf(buf, sizeof buf, "%s", s);
  if (!strchr(buf, 'T')) { char* sp = strchr(buf, ' '); if (sp) *sp = 'T'; }
  size_t bl = strlen(buf);
  if (bl > 0 && buf[bl - 1] == 'Z') buf[bl - 1] = 0;
  int y = 0, mo = 0, d = 0, h = 0, mi = 0, se = 0;
  if (sscanf(buf, "%d-%d-%d", &y, &mo, &d) != 3) return 0;
  char* t = strchr(buf, 'T');
  if (t && sscanf(t + 1, "%d:%d:%d", &h, &mi, &se) != 3) return 0;
  struct tm tmv;
  memset(&tmv, 0, sizeof tmv);
  tmv.tm_year = y - 1900; tmv.tm_mon = mo - 1; tmv.tm_mday = d;
  tmv.tm_hour = h; tmv.tm_min = mi; tmv.tm_sec = se;
#if defined(_WIN32)
  return (int64_t)_mkgmtime(&tmv);
#else
  return (int64_t)timegm(&tmv);
#endif
}
static Val _str_split(const char* s, const char* delim) {
  if (!s) return _arrn(NULL, 0);
  if (!delim || !delim[0]) {
    size_t n = strlen(s);
    Val* xs = (Val*)malloc(sizeof(Val) * (n + 1));
    for (size_t i = 0; i < n; i++) {
      char c[2] = { s[i], 0 };
      xs[i] = _v_str(c);
    }
    Val v = _arrn(xs, n);
    free(xs);
    return v;
  }
  size_t dl = strlen(delim), n = 0;
  Val* xs = NULL;
  int cap = 0;
  const char* p = s;
  while (1) {
    const char* q = strstr(p, delim);
    if (n >= cap) { cap = cap ? cap * 2 : 8; xs = (Val*)realloc(xs, sizeof(Val) * cap); }
    if (q) {
      size_t ln = (size_t)(q - p);
      char* m = (char*)malloc(ln + 1);
      memcpy(m, p, ln); m[ln] = 0;
      xs[n++] = _v_str(m);
      p = q + dl;
    } else {
      xs[n++] = _v_str(p);
      break;
    }
  }
  Val v = _arrn(xs, n);
  free(xs);
  return v;
}
static char* _time_fmt(int64_t ts, const char* fmt) {
  time_t t = (time_t)ts;
  struct tm tmv;
#if defined(_WIN32)
  struct tm* tp = gmtime_s(&tmv, &t) == 0 ? &tmv : NULL;
#else
  struct tm* tp = gmtime_r(&t, &tmv);
#endif
  if (!tp) { char* b = (char*)malloc(32); snprintf(b, 32, "%lldT00:00:00Z", (long long)ts); return b; }
  char buf[128];
  if (fmt && fmt[0] && strcmp(fmt, "%Y-%m-%dT%H:%M:%SZ")) {
    strftime(buf, sizeof buf, fmt, tp);
  } else {
    strftime(buf, sizeof buf, "%Y-%m-%dT%H:%M:%SZ", tp);
  }
  char* m = (char*)malloc(strlen(buf) + 1);
  strcpy(m, buf);
  return m;
}
static Val _env_list(void) {
#if !defined(_WIN32) && !defined(__APPLE__)
  int n = 0;
  while (environ[n]) n++;
  Val* xs = (Val*)malloc(sizeof(Val) * (n > 0 ? n : 1));
  for (int i = 0; i < n; i++) xs[i] = _v_str(environ[i]);
  Val v = _arrn(xs, n);
  free(xs);
  return v;
#else
  (void)0;
  return _arrn(NULL, 0);
#endif
}
static Val _fs_list(const char* path) {
#if !defined(_WIN32) && !defined(__APPLE__)
  DIR* d = opendir(path);
  if (!d) return _v_void();
  Val* xs = NULL;
  int n = 0, cap = 0;
  struct dirent* e;
  while ((e = readdir(d)) != NULL) {
    if (!strcmp(e->d_name, ".") || !strcmp(e->d_name, "..")) continue;
    if (n >= cap) { cap = cap ? cap * 2 : 16; xs = (Val*)realloc(xs, sizeof(Val) * cap); }
    xs[n++] = _v_str(e->d_name);
  }
  closedir(d);
  Val v = _arrn(xs, n);
  free(xs);
  return v;
#else
  (void)path;
  return _v_void();
#endif
}
static char* _trim(const char* s) {
  const char* p = s;
  while (*p == ' ' || *p == '\t' || *p == '\n' || *p == '\r' || *p == '\v' || *p == '\f') p++;
  size_t n = strlen(p);
  while (n > 0 && (p[n-1] == ' ' || p[n-1] == '\t' || p[n-1] == '\n' || p[n-1] == '\r' || p[n-1] == '\v' || p[n-1] == '\f')) n--;
  char* m = (char*)malloc(n + 1);
  memcpy(m, p, n); m[n] = 0;
  return m;
}
static char* _sub(const char* s, int64_t st, int64_t en) {
  int64_t n = (int64_t)strlen(s);
  if (st < 0) st = 0; if (st > n) st = n;
  if (en < 0) en = n; if (en > n) en = n;
  if (en < st) en = st;
  char* m = (char*)malloc((size_t)(en - st) + 1);
  memcpy(m, s + st, (size_t)(en - st)); m[en - st] = 0;
  return m;
}
static Val _to_chars(const char* s) {
  size_t n = strlen(s);
  Val* xs = (Val*)malloc(sizeof(Val) * (n + 1));
  for (size_t i = 0; i < n; i++) { char c[2] = { s[i], 0 }; xs[i] = _v_str(c); }
  Val v = _arrn(xs, n); free(xs);
  return v;
}
static Val _str_codes(const char* s) {
  size_t n = strlen(s);
  Val* xs = (Val*)malloc(sizeof(Val) * (n + 1));
  for (size_t i = 0; i < n; i++) xs[i] = _v_int((unsigned char)s[i]);
  Val v = _arrn(xs, n); free(xs);
  return v;
}
static char* _replace(const char* s, const char* from, const char* to) {
  if (!from || !from[0]) { char* m = (char*)malloc(strlen(s) + 1); strcpy(m, s); return m; }
  size_t fl = strlen(from), tl = strlen(to), n = strlen(s), cap = n + 1, ln = 0;
  char* out = (char*)malloc(cap);
  const char* p = s;
  for (;;) {
    const char* q = strstr(p, from);
    if (!q) {
      size_t rem = strlen(p);
      if (ln + rem + 1 > cap) { cap = ln + rem + 1; out = (char*)realloc(out, cap); }
      memcpy(out + ln, p, rem); ln += rem;
      break;
    }
    size_t pre = (size_t)(q - p);
    if (ln + pre + tl + 1 > cap) { cap = (ln + pre + tl + 1) * 2; out = (char*)realloc(out, cap); }
    memcpy(out + ln, p, pre); ln += pre;
    memcpy(out + ln, to, tl); ln += tl;
    p = q + fl;
  }
  out[ln] = 0;
  return out;
}
static Val _concat_list(Val list) {
  size_t cap = 64, ln = 0;
  char* out = (char*)malloc(cap);
  for (int i = 0; i < list.argc; i++) {
    const char* x = _fmt(list.items[i]);
    size_t xl = strlen(x);
    if (ln + xl + 1 > cap) { cap = (ln + xl + 1) * 2; out = (char*)realloc(out, cap); }
    memcpy(out + ln, x, xl); ln += xl;
  }
  out[ln] = 0;
  return _v_str(out);
}
static Val _buf_write(const char* path, const char* content) {
  FILE* f = fopen(path, "wb");
  if (!f) return _res(_v_str("no se pudo escribir"), 0);
  fwrite(content, 1, strlen(content), f);
  fclose(f);
  return _res(_v_bool(1), 1);
}

/* FFI minimo (system/getenv/putenv/getpid/memoria) */
static int64_t _ffi_ptr_alloc(size_t n) { return (int64_t)(intptr_t)malloc(n ? n : 1); }
static Val _ffi_call(Val h, const char* nm, Val args, const char* ret) {
  (void)h; (void)ret;
  if (!strcmp(nm, "system")) {
    if (args.argc > 0) return _v_int(system(args.items[args.argc - 1].s ? args.items[args.argc - 1].s : ""));
    return _v_int(0);
  }
#if defined(_WIN32)
  if (!strcmp(nm, "_getpid")) return _v_int((int64_t)_getpid());
#else
  if (!strcmp(nm, "_getpid")) return _v_int((int64_t)getpid());
#endif
  if (!strcmp(nm, "getenv")) {
    if (args.argc > 0) return _v_int((int64_t)(intptr_t)getenv(args.items[0].s ? args.items[0].s : ""));
    return _v_int(0);
  }
#if defined(_WIN32)
  if (!strcmp(nm, "_putenv")) {
    if (args.argc > 0) return _v_int(_putenv(args.items[0].s ? args.items[0].s : ""));
    return _v_int(0);
  }
#else
  if (!strcmp(nm, "_putenv")) {
    if (args.argc > 0) {
      /* putenv takes char* not const char*; macOS/Linux both need mutable string */
      char* env_str = args.items[0].s ? (char*)args.items[0].s : "";
      return _v_int(putenv(env_str));
    }
    return _v_int(0);
  }
#endif
  return _v_int(0);
}

/* corutinas (reanudables, un hilo por corutina, un solo hilo activo a la vez) */
static int _coro_seq = 0;
static const char* _coro_names[256];
static char _coro_done[256];
static volatile int _coro_active = -1;
static Val _call_by_name(const char* nm);
#if defined(_WIN32)
static HANDLE _coro_ev_resume[256];
static HANDLE _coro_ev_cede[256];
static DWORD WINAPI _coro_thread(LPVOID p) {
  int i = (int)(intptr_t)p;
  for (;;) {
    WaitForSingleObject(_coro_ev_resume[i], INFINITE);
    ResetEvent(_coro_ev_resume[i]);
    if (_coro_done[i]) break;
    _coro_active = i;
    Val r = _call_by_name(_coro_names[i]);
    if (_coro_active == i) { _coro_active = -1; _coro_done[i] = 1; }
    (void)r;
    SetEvent(_coro_ev_cede[i]);
  }
  return 0;
}
static Val _coro_create(const char* nm) {
  char buf[16];
  snprintf(buf, sizeof buf, "coro_%d", _coro_seq);
  if (_coro_seq < 256) {
    _coro_names[_coro_seq] = nm;
    _coro_done[_coro_seq] = 0;
    _coro_ev_resume[_coro_seq] = CreateEvent(NULL, TRUE, FALSE, NULL);
    _coro_ev_cede[_coro_seq] = CreateEvent(NULL, TRUE, FALSE, NULL);
    HANDLE h = CreateThread(NULL, 0, _coro_thread, (LPVOID)(intptr_t)_coro_seq, 0, NULL);
    if (h) CloseHandle(h);
    _coro_seq++;
  }
  return _v_str(buf);
}
static Val _coro_resume(const char* id) {
  for (int i = 0; i < _coro_seq; i++) {
    char cid[16];
    snprintf(cid, sizeof cid, "coro_%d", i);
    if (!strcmp(cid, id)) {
      if (_coro_done[i]) return _v_void();
      SetEvent(_coro_ev_resume[i]);
      WaitForSingleObject(_coro_ev_cede[i], INFINITE);
      ResetEvent(_coro_ev_cede[i]);
      return _v_void();
    }
  }
  return _v_void();
}
static Val _coro_cede(void) {
  if (_coro_active >= 0) {
    int i = _coro_active;
    _coro_active = -1;
    SetEvent(_coro_ev_cede[i]);
    WaitForSingleObject(_coro_ev_resume[i], INFINITE);
    ResetEvent(_coro_ev_resume[i]);
    _coro_active = i;
  }
  return _v_void();
}
#else
static pthread_t _coro_thr[256];
static pthread_mutex_t _coro_mx[256];
static pthread_cond_t _coro_cv_resume[256];
static pthread_cond_t _coro_cv_cede[256];
static volatile int _coro_want[256];
static volatile int _coro_yielded[256];
static void* _coro_thread(void* p) {
  int i = (int)(intptr_t)p;
  for (;;) {
    pthread_mutex_lock(&_coro_mx[i]);
    while (!_coro_want[i]) pthread_cond_wait(&_coro_cv_resume[i], &_coro_mx[i]);
    _coro_want[i] = 0;
    int done = _coro_done[i];
    pthread_mutex_unlock(&_coro_mx[i]);
    if (done) break;
    _coro_active = i;
    Val r = _call_by_name(_coro_names[i]);
    if (_coro_active == i) { _coro_active = -1; _coro_done[i] = 1; }
    (void)r;
    pthread_mutex_lock(&_coro_mx[i]);
    _coro_yielded[i] = 1;
    pthread_cond_signal(&_coro_cv_cede[i]);
    pthread_mutex_unlock(&_coro_mx[i]);
  }
  return NULL;
}
static Val _coro_create(const char* nm) {
  char buf[16];
  snprintf(buf, sizeof buf, "coro_%d", _coro_seq);
  if (_coro_seq < 256) {
    _coro_names[_coro_seq] = nm;
    _coro_done[_coro_seq] = 0;
    _coro_want[_coro_seq] = 0;
    _coro_yielded[_coro_seq] = 0;
    pthread_mutex_init(&_coro_mx[_coro_seq], NULL);
    pthread_cond_init(&_coro_cv_resume[_coro_seq], NULL);
    pthread_cond_init(&_coro_cv_cede[_coro_seq], NULL);
    pthread_create(&_coro_thr[_coro_seq], NULL, _coro_thread, (void*)(intptr_t)_coro_seq);
    _coro_seq++;
  }
  return _v_str(buf);
}
static Val _coro_resume(const char* id) {
  for (int i = 0; i < _coro_seq; i++) {
    char cid[16];
    snprintf(cid, sizeof cid, "coro_%d", i);
    if (!strcmp(cid, id)) {
      if (_coro_done[i]) return _v_void();
      pthread_mutex_lock(&_coro_mx[i]);
      _coro_want[i] = 1;
      pthread_cond_signal(&_coro_cv_resume[i]);
      while (!_coro_yielded[i]) pthread_cond_wait(&_coro_cv_cede[i], &_coro_mx[i]);
      _coro_yielded[i] = 0;
      pthread_mutex_unlock(&_coro_mx[i]);
      return _v_void();
    }
  }
  return _v_void();
}
static Val _coro_cede(void) {
  if (_coro_active >= 0) {
    int i = _coro_active;
    _coro_active = -1;
    pthread_mutex_lock(&_coro_mx[i]);
    _coro_yielded[i] = 1;
    pthread_cond_signal(&_coro_cv_cede[i]);
    while (!_coro_want[i]) pthread_cond_wait(&_coro_cv_resume[i], &_coro_mx[i]);
    _coro_want[i] = 0;
    pthread_mutex_unlock(&_coro_mx[i]);
    _coro_active = i;
  }
  return _v_void();
}
#endif

/* SHA-256 */
#define ROTR32(x,n) (((x) >> (n)) | ((x) << (32 - (n))))
static const uint32_t _K256[64] = {
  0x428a2f98,0x71374491,0xb5c0fbcf,0xe9b5dba5,0x3956c25b,0x59f111f1,0x923f82a4,0xab1c5ed5,
  0xd807aa98,0x12835b01,0x243185be,0x550c7dc3,0x72be5d74,0x80deb1fe,0x9bdc06a7,0xc19bf174,
  0xe49b69c1,0xefbe4786,0x0fc19dc6,0x240ca1cc,0x2de92c6f,0x4a7484aa,0x5cb0a9dc,0x76f988da,
  0x983e5152,0xa831c66d,0xb00327c8,0xbf597fc7,0xc6e00bf3,0xd5a79147,0x06ca6351,0x14292967,
  0x27b70a85,0x2e1b2138,0x4d2c6dfc,0x53380d13,0x650a7354,0x766a0abb,0x81c2c92e,0x92722c85,
  0xa2bfe8a1,0xa81a664b,0xc24b8b70,0xc76c51a3,0xd192e819,0xd6990624,0xf40e3585,0x106aa070,
  0x19a4c116,0x1e376c08,0x2748774c,0x34b0bcb5,0x391c0cb3,0x4ed8aa4a,0x5b9cca4f,0x682e6ff3,
  0x748f82ee,0x78a5636f,0x84c87814,0x8cc70208,0x90befffa,0xa4506ceb,0xbef9a3f7,0xc67178f2
};
static void _sha256(const unsigned char* d, size_t n, unsigned char* out) {
  uint32_t h[8] = {0x6a09e667,0xbb67ae85,0x3c6ef372,0xa54ff53a,0x510e527f,0x9b05688c,0x1f83d9ab,0x5be0cd19};
  size_t ml = n + 1;
  while (ml % 64 != 56) ml++;
  ml += 8;
  unsigned char* m = (unsigned char*)calloc(ml, 1);
  memcpy(m, d, n);
  m[n] = 0x80;
  uint64_t bits = (uint64_t)n * 8;
  for (int i = 0; i < 8; i++) m[ml - 1 - i] = (unsigned char)(bits >> (8 * i));
  for (size_t off = 0; off < ml; off += 64) {
    uint32_t w[64];
    for (int i = 0; i < 16; i++) w[i] = ((uint32_t)m[off + 4 * i] << 24) | ((uint32_t)m[off + 4 * i + 1] << 16) | ((uint32_t)m[off + 4 * i + 2] << 8) | m[off + 4 * i + 3];
    for (int i = 16; i < 64; i++) {
      uint32_t s0 = ROTR32(w[i - 15], 7) ^ ROTR32(w[i - 15], 18) ^ (w[i - 15] >> 3);
      uint32_t s1 = ROTR32(w[i - 2], 17) ^ ROTR32(w[i - 2], 19) ^ (w[i - 2] >> 10);
      w[i] = w[i - 16] + s0 + w[i - 7] + s1;
    }
    uint32_t a = h[0], b = h[1], c = h[2], dd = h[3], e = h[4], f = h[5], g = h[6], hh = h[7];
    for (int i = 0; i < 64; i++) {
      uint32_t S1 = ROTR32(e, 6) ^ ROTR32(e, 11) ^ ROTR32(e, 25);
      uint32_t ch = (e & f) ^ (~e & g);
      uint32_t t1 = hh + S1 + ch + _K256[i] + w[i];
      uint32_t S0 = ROTR32(a, 2) ^ ROTR32(a, 13) ^ ROTR32(a, 22);
      uint32_t maj = (a & b) ^ (a & c) ^ (b & c);
      uint32_t t2 = S0 + maj;
      hh = g; g = f; f = e; e = dd + t1; dd = c; c = b; b = a; a = t1 + t2;
    }
    h[0] += a; h[1] += b; h[2] += c; h[3] += dd; h[4] += e; h[5] += f; h[6] += g; h[7] += hh;
  }
  free(m);
  for (int i = 0; i < 8; i++) {
    out[4 * i] = (unsigned char)(h[i] >> 24); out[4 * i + 1] = (unsigned char)(h[i] >> 16);
    out[4 * i + 2] = (unsigned char)(h[i] >> 8); out[4 * i + 3] = (unsigned char)h[i];
  }
}

/* SHA-512 */
#define ROTR64(x,n) (((x) >> (n)) | ((x) << (64 - (n))))
static const uint64_t _K512[80] = {
  0x428a2f98d728ae22ULL,0x7137449123ef65cdULL,0xb5c0fbcfec4d3b2fULL,0xe9b5dba58189dbbcULL,0x3956c25bf348b538ULL,0x59f111f1b605d019ULL,0x923f82a4af194f9bULL,0xab1c5ed5da6d8118ULL,
  0xd807aa98a3030242ULL,0x12835b0145706fbeULL,0x243185be4ee4b28cULL,0x550c7dc3d5ffb4e2ULL,0x72be5d74f27b896fULL,0x80deb1fe3b1696b1ULL,0x9bdc06a725c71235ULL,0xc19bf174cf692694ULL,
  0xe49b69c19ef14ad2ULL,0xefbe4786384f25e3ULL,0x0fc19dc68b8cd5b5ULL,0x240ca1cc77ac9c65ULL,0x2de92c6f592b0275ULL,0x4a7484aa6ea6e483ULL,0x5cb0a9dcbd41fbd4ULL,0x76f988da831153b5ULL,
  0x983e5152ee66dfabULL,0xa831c66d2db43210ULL,0xb00327c898fb213fULL,0xbf597fc7beef0ee4ULL,0xc6e00bf33da88fc2ULL,0xd5a79147930aa725ULL,0x06ca6351e003826fULL,0x142929670a0e6e70ULL,
  0x27b70a8546d22ffcULL,0x2e1b21385c26c926ULL,0x4d2c6dfc5ac42aedULL,0x53380d139d95b3dfULL,0x650a73548baf63deULL,0x766a0abb3c77b2a8ULL,0x81c2c92e47edaee6ULL,0x92722c851482353bULL,
  0xa2bfe8a14cf10364ULL,0xa81a664bbc423001ULL,0xc24b8b70d0f89791ULL,0xc76c51a30654be30ULL,0xd192e819d6ef5218ULL,0xd69906245565a910ULL,0xf40e35855771202aULL,0x106aa07032bbd1b8ULL,
  0x19a4c116b8d2d0c8ULL,0x1e376c085141ab53ULL,0x2748774cdf8eeb99ULL,0x34b0bcb5e19b48a8ULL,0x391c0cb3c5c95a63ULL,0x4ed8aa4ae3418acbULL,0x5b9cca4f7763e373ULL,0x682e6ff3d6b2b8a3ULL,
  0x748f82ee5defb2fcULL,0x78a5636f43172f60ULL,0x84c87814a1f0ab72ULL,0x8cc702081a6439ecULL,0x90befffa23631e28ULL,0xa4506cebde82bde9ULL,0xbef9a3f7b2c67915ULL,0xc67178f2e372532bULL,
  0xca273eceea26619cULL,0xd186b8c721c0c207ULL,0xeada7dd6cde0eb1eULL,0xf57d4f7fee6ed178ULL,0x06f067aa72176fbaULL,0x0a637dc5a2c898a6ULL,0x113f9804bef90daeULL,0x1b710b35131c471bULL,
  0x28db77f523047d84ULL,0x32caab7b40c72493ULL,0x3c9ebe0a15c9bebcULL,0x431d67c49c100d4cULL,0x4cc5d4becb3e42b6ULL,0x597f299cfc657e2aULL,0x5fcb6fab3ad6faecULL,0x6c44198c4a475817ULL
};
static void _sha512(const unsigned char* d, size_t n, unsigned char* out) {
  uint64_t h[8] = {0x6a09e667f3bcc908ULL,0xbb67ae8584caa73bULL,0x3c6ef372fe94f82bULL,0xa54ff53a5f1d36f1ULL,0x510e527fade682d1ULL,0x9b05688c2b3e6c1fULL,0x1f83d9abfb41bd6bULL,0x5be0cd19137e2179ULL};
  size_t ml = n + 1;
  while (ml % 128 != 112) ml++;
  ml += 16;
  unsigned char* m = (unsigned char*)calloc(ml, 1);
  memcpy(m, d, n);
  m[n] = 0x80;
  uint64_t bits = (uint64_t)n * 8;
  for (int i = 0; i < 8; i++) m[ml - 1 - i] = (unsigned char)(bits >> (8 * i));
  for (size_t off = 0; off < ml; off += 128) {
    uint64_t w[80];
    for (int i = 0; i < 16; i++) {
      w[i] = 0;
      for (int j = 0; j < 8; j++) w[i] = (w[i] << 8) | m[off + 8 * i + j];
    }
    for (int i = 16; i < 80; i++) {
      uint64_t s0 = ROTR64(w[i - 15], 1) ^ ROTR64(w[i - 15], 8) ^ (w[i - 15] >> 7);
      uint64_t s1 = ROTR64(w[i - 2], 19) ^ ROTR64(w[i - 2], 61) ^ (w[i - 2] >> 6);
      w[i] = w[i - 16] + s0 + w[i - 7] + s1;
    }
    uint64_t a = h[0], b = h[1], c = h[2], dd = h[3], e = h[4], f = h[5], g = h[6], hh = h[7];
    for (int i = 0; i < 80; i++) {
      uint64_t S1 = ROTR64(e, 14) ^ ROTR64(e, 18) ^ ROTR64(e, 41);
      uint64_t ch = (e & f) ^ (~e & g);
      uint64_t t1 = hh + S1 + ch + _K512[i] + w[i];
      uint64_t S0 = ROTR64(a, 28) ^ ROTR64(a, 34) ^ ROTR64(a, 39);
      uint64_t maj = (a & b) ^ (a & c) ^ (b & c);
      uint64_t t2 = S0 + maj;
      hh = g; g = f; f = e; e = dd + t1; dd = c; c = b; b = a; a = t1 + t2;
    }
    h[0] += a; h[1] += b; h[2] += c; h[3] += dd; h[4] += e; h[5] += f; h[6] += g; h[7] += hh;
  }
  free(m);
  for (int i = 0; i < 8; i++) {
    for (int j = 0; j < 8; j++) out[8 * i + j] = (unsigned char)(h[i] >> (56 - 8 * j));
  }
}
static char* _hash_hex(const char* s, int bits) {
  size_t n = strlen(s);
  unsigned char dig[64];
  if (bits == 256) _sha256((const unsigned char*)s, n, dig);
  else _sha512((const unsigned char*)s, n, dig);
  int dl = bits / 8;
  char* out = (char*)malloc((size_t)dl * 2 + 1);
  for (int i = 0; i < dl; i++) snprintf(out + i * 2, 3, "%02x", dig[i]);
  return out;
}

/* JSON minimo (objetos/arrays/strings/numeros/bools/null) */
static const char* _jp;
static void _js_skip(void) { while (*_jp == ' ' || *_jp == '\t' || *_jp == '\n' || *_jp == '\r') _jp++; }
static char* _js_str(void) {
  _jp++;
  size_t cap = 64, len = 0;
  char* b = (char*)malloc(cap);
  while (*_jp && *_jp != '"') {
    char c = *_jp;
    if (c == '\\') {
      _jp++;
      switch (*_jp) {
        case 'n': c = '\n'; break;
        case 't': c = '\t'; break;
        case 'r': c = '\r'; break;
        case 'b': c = '\b'; break;
        case 'f': c = '\f'; break;
        case '/': c = '/'; break;
        case '"': c = '"'; break;
        case '\\': c = '\\'; break;
        case 'u': {
          if (_jp[1] && _jp[2] && _jp[3] && _jp[4]) {
            uint32_t cp = (uint32_t)strtol(_jp + 1, NULL, 16);
            unsigned char u8[4];
            size_t nn = _u8e(cp, u8);
            if (len + nn + 1 > cap) { cap *= 2; b = (char*)realloc(b, cap); }
            memcpy(b + len, u8, nn); len += nn;
            _jp += 4;
          }
          c = 0;
          break;
        }
        default: c = *(_jp = _jp - 1); break;
      }
    }
    if (c) {
      if (len + 2 > cap) { cap *= 2; b = (char*)realloc(b, cap); }
      b[len++] = c;
    }
    _jp++;
  }
  if (*_jp == '"') _jp++;
  b[len] = 0;
  return b;
}
static Val _js_value(void);
static Val _js_value(void) {
  _js_skip();
  if (*_jp == '{') {
    _jp++;
    Val m = _map_new();
    while (1) {
      _js_skip();
      if (*_jp == '}') { _jp++; return m; }
      char* k = _js_str();
      _js_skip();
      if (*_jp == ':') _jp++;
      Val v = _js_value();
      m = _map_set(m, _v_str(k), v);
      _js_skip();
      if (*_jp == ',') { _jp++; continue; }
      if (*_jp == '}') { _jp++; return m; }
      return m;
    }
  }
  if (*_jp == '[') {
    _jp++;
    Val* xs = NULL;
    int n = 0, cap = 0;
    while (1) {
      _js_skip();
      if (*_jp == ']') { _jp++; break; }
      if (n >= cap) { cap = cap ? cap * 2 : 8; xs = (Val*)realloc(xs, sizeof(Val) * cap); }
      xs[n++] = _js_value();
      _js_skip();
      if (*_jp == ',') { _jp++; continue; }
      if (*_jp == ']') { _jp++; break; }
      break;
    }
    Val v = _arrn(xs, n);
    free(xs);
    return v;
  }
  if (*_jp == '"') return _v_str(_js_str());
  if (!strncmp(_jp, "true", 4)) { _jp += 4; return _v_bool(1); }
  if (!strncmp(_jp, "false", 5)) { _jp += 5; return _v_bool(0); }
  if (!strncmp(_jp, "null", 4)) { _jp += 4; return _v_void(); }
  char* end = NULL;
  double dv = strtod(_jp, &end);
  _jp = end;
  if (dv == (double)(int64_t)dv && fabs(dv) < 9e15) return _v_int((int64_t)dv);
  return _v_flt(dv);
}
static Val _json_parse(const char* s) { _jp = s; return _js_value(); }
static const char* _jesc(const char* s) { return s; }
static char* _json_text(Val v) {
  char* b = (char*)malloc(1024);
  size_t len = 0;
  switch (v.t) {
    case T_MAP: {
      int* idx = (int*)malloc(sizeof(int) * (v.argc > 0 ? v.argc : 1));
      for (int i = 0; i < v.argc; i++) idx[i] = i;
      for (int i = 0; i < v.argc; i++)
        for (int j = i + 1; j < v.argc; j++)
          if (v.items[idx[j] * 2].s && v.items[idx[i] * 2].s && strcmp(v.items[idx[j] * 2].s, v.items[idx[i] * 2].s) < 0) { int t2 = idx[i]; idx[i] = idx[j]; idx[j] = t2; }
      b[len++] = '{';
      for (int i = 0; i < v.argc; i++) {
        if (i > 0) b[len++] = ',';
        Val k = v.items[idx[i] * 2];
        b[len++] = '"';
        const char* ks = k.s ? k.s : "";
        memcpy(b + len, ks, strlen(ks)); len += strlen(ks);
        b[len++] = '"';
        b[len++] = ':';
        char* tv = _json_text(v.items[idx[i] * 2 + 1]);
        size_t tl = strlen(tv);
        if (len + tl + 16 > 1024) { size_t nc = len + tl + 32; char* nb = (char*)realloc(b, nc); b = nb; }
        memcpy(b + len, tv, tl); len += tl;
        free(tv);
      }
      b[len++] = '}';
      b[len] = 0;
      free(idx);
      break;
    }
    case T_ARR: {
      b[len++] = '[';
      for (int i = 0; i < v.argc; i++) {
        if (i > 0) b[len++] = ',';
        char* tv = _json_text(v.items[i]);
        size_t tl = strlen(tv);
        if (len + tl + 16 > 1024) { size_t nc = len + tl + 32; b = (char*)realloc(b, nc); }
        memcpy(b + len, tv, tl); len += tl;
        free(tv);
      }
      b[len++] = ']';
      b[len] = 0;
      break;
    }
    case T_STR: {
      b[len++] = '"';
      const char* s2 = v.s ? v.s : "";
      const char* p = s2;
      while (*p) {
        if (*p == '"' || *p == '\\') { b[len++] = '\\'; b[len++] = *p; }
        else b[len++] = *p;
        p++;
      }
      b[len++] = '"';
      b[len] = 0;
      break;
    }
    case T_INT:
      len = (size_t)snprintf(b, 1024, "%lld", (long long)v.i);
      break;
    case T_FLT: {
      char buf[64];
      if (v.f == (double)(int64_t)v.f && fabs(v.f) < 1e16) snprintf(buf, sizeof buf, "%lld", (long long)v.f);
      else snprintf(buf, sizeof buf, "%.17g", v.f);
      len = (size_t)snprintf(b, 1024, "%s", buf);
      break;
    }
    case T_BOL:
      len = (size_t)snprintf(b, 1024, "%s", v.i ? "true" : "false");
      break;
    default:
      len = (size_t)snprintf(b, 1024, "null");
      break;
  }
  (void)_jesc;
  return b;
}

#endif