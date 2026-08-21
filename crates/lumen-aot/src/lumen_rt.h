#ifndef LUMEN_RT_H
#define LUMEN_RT_H

#include <setjmp.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <math.h>
#include <time.h>

/* POSIX headers for non-Windows (Linux + macOS) */
#if !defined(_WIN32)
/* BUG-165: <sys/resource.h> estaba incluido ARRIBA, fuera de este bloque, asi
 * que TODO binario nativo generado en Windows fallaba a compilar con
 * "fatal error: sys/resource.h: No such file or directory". Solo se necesita
 * para getrlimit/setrlimit, que son POSIX. */
#include <sys/resource.h>
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
  /* BUG-032: entorno capturado por la closure (nombres + valores). Se llena
   * al crear la referencia a funcion y se vuelca en las globales justo antes
   * de invocarla, para que la closure siga siendo valida cuando el marco que
   * la creo ya no existe y dos instancias no compartan estado. */
  struct _Env* env;
  /* BUG-083: copy-on-write. 1 = `items` puede estar compartido con otro Val,
   * asi que hay que materializar una copia privada antes de mutarlo. */
  int shared;
} Val;

typedef struct _Env {
  int n;
  const char** names;
  struct Val* vals;
} _Env;

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

/* BUG-051: la pila de valores no tenia comprobacion de limites y las funciones
 * LUMEN se emiten como funciones C recursivas, asi que una recursion infinita
 * desbordaba la pila del proceso y moria por SEGFAULT silencioso (rc=139, sin
 * ningun mensaje). La VM ya aborta con un error legible (BUG-049); el binario
 * nativo debe comportarse igual. Se vigilan las dos cosas: el tope de ST[] y
 * la profundidad de la pila de C. */
/* La pila de valores crece bajo demanda: un programa con recursion legitima
 * profunda (p.ej. suma(100000)) necesita mucho mas de 16384 ranuras, y la VM
 * lo admite. Un tope fijo aqui habria roto la paridad VM/AOT. */
/* BUG-081: la pila de operandos y el control de profundidad eran globales
 * COMPARTIDOS entre hilos. Las corrutinas del backend C corren en hilos
 * pthread propios, asi que el hilo de la corrutina pisaba `SP` del principal y
 * medía su profundidad contra `_stack_base` de OTRA pila: el binario abortaba
 * con "pila agotada / recursion infinita" en cuanto se reanudaba una corrutina,
 * mientras la VM ejecutaba el programa entero sin problema. Cada hilo necesita
 * su propia pila de operandos y su propio contador. */
#if defined(_WIN32)
#define _LTLS __declspec(thread)
#else
#define _LTLS __thread
#endif
static _LTLS Val *ST = 0;
static _LTLS int ST_CAP = 0;
static _LTLS int SP = 0;
#define MAX_CALL_DEPTH 250000
static _LTLS int _depth = 0;
static _LTLS char *_stack_base = 0;
/* Margen de seguridad respecto al limite real de pila del hilo. */
static _LTLS size_t _stack_limit = 0;

/* BUG-022: manejadores de `intentar/atrapar`. El backend C usa `goto`, que no
 * puede saltar entre funciones, asi que el desenrollado se hace con
 * setjmp/longjmp: `_hnd_push` marca el punto de retorno y cualquier error del
 * runtime salta ahi en vez de llamar a exit(). */
#define MAX_HND 256
typedef struct {
  jmp_buf env;
  int sp;    /* altura de la pila de valores al instalarlo */
  int depth; /* profundidad de llamadas al instalarlo */
} _Hnd;
static _Hnd _hnd[MAX_HND];
static int _hnd_n = 0;
/* Mensaje del error en curso, para ligarlo a la variable del `atrapar`. */
static char _hnd_msg[512];

/* BUG-104: variante de `_rt_fatal` para division por cero, indice fuera de rango
 * y campo inexistente. Historicamente salia con codigo 3 mientras la VM salia
 * con 1 para EL MISMO error, asi que un script que comprobara el codigo de
 * salida veia dos lenguajes distintos segun el backend. Ahora ambos usan 1;
 * se conserva la funcion aparte porque su semantica de `atrapar` difiere
 * (liga el mensaje sin el prefijo "Error: "). */
static void _rt_error3(const char *msg) __attribute__((noreturn));

static void _rt_fatal(const char *msg) {
  /* Si hay un `atrapar` vigente, esto no es fatal: se desenrolla hasta el. */
  if (_hnd_n > 0) {
    snprintf(_hnd_msg, sizeof(_hnd_msg), "%s", msg);
    _Hnd *h = &_hnd[_hnd_n - 1];
    longjmp(h->env, 1);
  }
  fflush(stdout);
  fprintf(stderr, "Error: %s\n", msg);
  exit(1);
}

static void _rt_error3(const char *msg) {
  if (_hnd_n > 0) {
    /* Atrapado: el mensaje se liga tal cual a la variable del `atrapar`, igual
       que en la VM (sin el prefijo "Error: "). */
    snprintf(_hnd_msg, sizeof(_hnd_msg), "%s", msg);
    _Hnd *h = &_hnd[_hnd_n - 1];
    longjmp(h->env, 1);
  }
  fflush(stdout);
  fprintf(stderr, "Error: %s\n", msg);
  exit(1);
}

/* Duplica la capacidad de la pila de valores. Se aborta si el SO no puede dar
 * mas memoria, en vez de corromper el heap en silencio. */
static void _st_grow(void) {
  int _n = ST_CAP ? ST_CAP * 2 : 16384;
  Val *_p = (Val *)realloc(ST, (size_t)_n * sizeof(Val));
  if (!_p) {
    _rt_fatal("Sin memoria para la pila de valores. "
              "\xc2\xbfHay una recursi\xc3\xb3n infinita?");
  }
  ST = _p;
  ST_CAP = _n;
}
/* BUG-081: init de la pila de operandos para hilos secundarios (corrutinas).
   Un hilo pthread tiene una pila mucho menor que la del principal (8 MiB por
   defecto y sin crecimiento bajo demanda), asi que el margen se fija mas bajo. */
static void _coro_stack_init(void) {
  char _probe;
  _stack_base = &_probe;
  _depth = 0;
  if (!ST) _st_grow();
  _stack_limit = 4UL * 1024 * 1024;
}

#ifdef _WIN32
/* BUG-165: en Windows no hay getrlimit/RLIMIT_STACK. El tamano de pila se fija
 * al enlazar (1 MiB por defecto, no los 8 MiB tipicos de Linux), asi que aqui
 * se consulta el rango real del hilo en vez de intentar ampliarlo. Sin esto el
 * limite quedaba sin inicializar y la deteccion de desbordamiento —lo que
 * convierte un segfault mudo en un mensaje— no protegia nada. */
static void _stack_init(void) {
  char _probe;
  _stack_base = &_probe;
  if (!ST) _st_grow();
  _stack_limit = 1UL * 1024 * 1024 - 256UL * 1024; /* conservador por defecto */
#if defined(_WIN32_WINNT) && _WIN32_WINNT >= 0x0602
  {
    ULONG_PTR _low = 0, _high = 0;
    GetCurrentThreadStackLimits(&_low, &_high);
    if (_high > _low) {
      size_t _total = (size_t)(_high - _low);
      /* Deja ~256 KiB de reserva para poder formatear el error e imprimirlo. */
      _stack_limit = _total > 512UL * 1024 ? _total - 256UL * 1024 : _total / 2;
    }
  }
#endif
}
#else
static void _stack_init(void) {
  char _probe;
  struct rlimit _rl;
  _stack_base = &_probe;
  if (!ST) _st_grow();
  /* Un marco de funcion C gasta bastante mas pila que un marco de la VM, asi
   * que con los 8 MiB por defecto un programa con recursion legitima profunda
   * (suma(100000)) abortaba en nativo aunque la VM lo resolviera. Se sube el
   * limite blando hasta el duro: en Linux la pila del hilo principal crece
   * bajo demanda hasta RLIMIT_STACK, de modo que subirlo aqui ya da margen. */
  if (getrlimit(RLIMIT_STACK, &_rl) == 0) {
    rlim_t _want = (rlim_t)1024 * 1024 * 1024; /* 1 GiB */
    if (_rl.rlim_max != RLIM_INFINITY && _want > _rl.rlim_max) {
      _want = _rl.rlim_max;
    }
    if (_rl.rlim_cur != RLIM_INFINITY && _want > _rl.rlim_cur) {
      _rl.rlim_cur = _want;
      if (setrlimit(RLIMIT_STACK, &_rl) != 0) {
        (void)getrlimit(RLIMIT_STACK, &_rl);
      }
    }
  }
  _stack_limit = 6UL * 1024 * 1024;
  if (getrlimit(RLIMIT_STACK, &_rl) == 0) {
    if (_rl.rlim_cur == RLIM_INFINITY) {
      _stack_limit = (size_t)1024 * 1024 * 1024;
    } else if (_rl.rlim_cur > 2UL * 1024 * 1024) {
      /* Deja ~1 MiB de reserva para poder formatear el error e imprimirlo. */
      _stack_limit = (size_t)_rl.rlim_cur - 1024UL * 1024;
    }
  }
}
#endif /* _WIN32 */

/* Se llama al entrar en cada funcion LUMEN. */
static void _ckdepth(void) {
  char _probe;
  size_t _used;
  if (++_depth > MAX_CALL_DEPTH) {
    _rt_fatal("Profundidad m\xc3\xa1xima de llamadas superada (250000). "
              "\xc2\xbfHay una recursi\xc3\xb3n infinita?");
  }
  if (_stack_base) {
    _used = (size_t)(_stack_base > &_probe ? _stack_base - &_probe
                                           : &_probe - _stack_base);
    if (_used > _stack_limit) {
      _rt_fatal("Profundidad m\xc3\xa1xima de llamadas superada "
                "(pila agotada). \xc2\xbfHay una recursi\xc3\xb3n infinita?");
    }
  }
}

#define PUSH(v) (((SP) >= ST_CAP ? _st_grow() : (void)0), ST[(SP)++] = (v))
#define POP() (ST[--(SP)])
#define TOP() (ST[SP - 1])

static Val gv[16384];
static const char* gn[16384];
static int gc = 0;

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
  v.env = 0;
  return v;
}

/* BUG-032: captura por valor de los nombres indicados, en el momento de crear
 * la closure. Cada `_vfclos` produce un entorno propio. */
static Val _vfclos(const char* n, Val (*fp)(void), const char** names, int cnt) {
  Val v = _vfref(n, fp);
  if (cnt > 0) {
    _Env* e = (_Env*)malloc(sizeof(_Env));
    e->n = cnt;
    e->names = (const char**)malloc((size_t)cnt * sizeof(char*));
    e->vals = (Val*)malloc((size_t)cnt * sizeof(Val));
    for (int i = 0; i < cnt; i++) {
      e->names[i] = names[i];
      e->vals[i] = gv[_fv(names[i])];
    }
    v.env = e;
  }
  return v;
}

/* BUG-149: al volver de una closure, el llamador restaura sus propias
 * variables (BUG-061) para que una lambda recursiva no se pise a si misma.
 * Esa restauracion tambien deshacia, sin querer, la mutacion que la closure
 * acababa de hacer sobre una variable CAPTURADA: `inc(5)` dejaba x=5 y el
 * restore lo devolvia a 0. Para las variables que la closure capturo, el
 * valor bueno es el de su entorno, no el que el llamador guardo. */
static Val _env_or(Val cf, const char* name, Val saved) {
  if (cf.env) {
    _Env* e = cf.env;
    for (int i = 0; i < e->n; i++)
      if (!strcmp(e->names[i], name)) return e->vals[i];
  }
  return saved;
}

static Val _fref_call(Val v) {
  if (!v.fp) return _v_void();
  if (v.env) {
    /* Restaura el entorno capturado y lo deshace al volver, para no pisar las
     * variables homonimas del llamador. */
    _Env* e = v.env;
    Val* saved = (Val*)malloc((size_t)e->n * sizeof(Val));
    int* slots = (int*)malloc((size_t)e->n * sizeof(int));
    Val r;
    for (int i = 0; i < e->n; i++) {
      slots[i] = _fv(e->names[i]);
      saved[i] = gv[slots[i]];
      gv[slots[i]] = e->vals[i];
    }
    r = v.fp();
    /* BUG-052: antes de deshacer, guarda el valor final en el entorno de ESTA
     * closure, para que `n = n + 1` persista de una llamada a la siguiente. El
     * entorno es propio de cada instancia, asi que no se contaminan entre si. */
    for (int i = 0; i < e->n; i++) e->vals[i] = gv[slots[i]];
    /* BUG-149: la restauracion incondicional del valor previo descartaba la
     * mutacion de cara al entorno que declara la variable: la closure veia su
     * propio estado avanzar (5, 10, 15) mientras la variable original seguia
     * en 0. La captura es UNA sola variable, no dos copias divergentes, asi
     * que el valor final se propaga a la global, igual que hace la VM. El
     * aislamiento entre instancias que introdujo BUG-032 lo sigue dando
     * `e->vals`, que es propio de cada closure: al entrar, cada una reinstala
     * su estado, de modo que dos closures de la misma factoria no se pisan.
     * `saved` deja de usarse para restaurar, pero se conserva para las
     * variables que la closure NO capturo. */
    for (int i = 0; i < e->n; i++) gv[slots[i]] = e->vals[i];
    (void)saved;
    free(saved);
    free(slots);
    return r;
  }
  return v.fp();
}

static int _isnum(Val v) { return v.t == T_INT || v.t == T_FLT || v.t == T_BOL; }
static double _asf(Val v) { return v.t == T_FLT ? v.f : (double)v.i; }

/* ── Conversiones y matemáticas públicas (BUG-001 / BUG-002 / BUG-007) ──
   Réplica exacta de parse_int_value / parse_float_value de la VM para que el
   binario nativo y el intérprete den el mismo resultado. */
static int _parse_f64(Val v, double* out) {
  if (v.t == T_FLT) { *out = v.f; return 1; }
  if (v.t == T_INT || v.t == T_BOL) { *out = (double)v.i; return 1; }
  if (v.t == T_STR && v.s) {
    const char* p = v.s;
    while (*p == ' ' || *p == '\t' || *p == '\n' || *p == '\r') p++;
    if (!*p) return 0;
    char* end = NULL;
    double d = strtod(p, &end);
    if (end == p) return 0;
    while (*end == ' ' || *end == '\t' || *end == '\n' || *end == '\r') end++;
    if (*end) return 0;
    *out = d;
    return 1;
  }
  return 0;
}

/* BUG-114: convertir a int64 un `double` que no cabe en el rango es
   comportamiento indefinido en C: `a_entero(1.0e300)` devolvia
   -9223372036854775808 en el binario nativo mientras la VM devolvia
   9223372036854775807. El `as i64` de Rust —que el comentario original decia
   imitar— SATURA a los extremos y convierte NaN en 0. Se replica aqui. */
static int64_t _f2i_sat(double d) {
  if (isnan(d)) return 0;
  if (d >= 9223372036854775808.0) return INT64_MAX;
  if (d <= -9223372036854775808.0) return INT64_MIN;
  return (int64_t)d; /* trunca hacia cero */
}

static Val _b_a_entero(Val v) {
  double d;
  if (!_parse_f64(v, &d)) return _v_int(0);
  return _v_int(_f2i_sat(d));
}

static Val _b_a_decimal(Val v) {
  double d;
  if (!_parse_f64(v, &d)) return _v_flt(0.0);
  return _v_flt(d);
}

static Val _b_es_numero(Val v) {
  double d;
  return _v_bool(_parse_f64(v, &d));
}

static Val _b_abs(Val v) {
  if (v.t == T_INT) return _v_int(v.i < 0 ? -v.i : v.i);
  double d;
  if (!_parse_f64(v, &d)) return _v_int(0);
  return _v_flt(fabs(d));
}

/* want_max != 0 → maximo(); preserva entero cuando ambos lo son. */
static Val _b_minmax(Val a, Val b, int want_max) {
  if (a.t == T_INT && b.t == T_INT)
    return _v_int(((a.i > b.i) == (want_max != 0)) ? a.i : b.i);
  double x = 0.0, y = 0.0;
  _parse_f64(a, &x);
  _parse_f64(b, &y);
  return _v_flt(((x > y) == (want_max != 0)) ? x : y);
}

/* modo: 0 = raiz, 1 = piso, 2 = techo, 3 = redondear */
static Val _b_math1(Val v, int mode) {
  double d = 0.0;
  _parse_f64(v, &d);
  switch (mode) {
    case 0: return _v_flt(sqrt(d));
    case 1: return _v_flt(floor(d));
    case 2: return _v_flt(ceil(d));
    default: return _v_flt(round(d));
  }
}

static Val _b_potencia(Val b, Val e) {
  double x = 0.0, y = 0.0;
  _parse_f64(b, &x);
  _parse_f64(e, &y);
  return _v_flt(pow(x, y));
}

static inline int _eq(Val a, Val b) {
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
    /* BUG-038: los structs no tenían rama y caían en `default`, que compara
       `a.i` — basura para un struct — dando `true` para structs distintos.
       La VM sí los compara por contenido (BUG-030). Los campos se guardan
       intercalados nombre/valor, de ahí el paso de 2 en 2. */
    case T_STT: {
      if (a.en && b.en && strcmp(a.en, b.en) != 0) return 0;
      if (a.argc != b.argc) return 0;
      for (int i = 0; i < a.argc; i++) {
        if (strcmp(a.items[2 * i].s, b.items[2 * i].s) != 0) return 0;
        if (!_eq(a.items[2 * i + 1], b.items[2 * i + 1])) return 0;
      }
      return 1;
    }
    /* BUG-038: `ninguno == ninguno` y `error(x) == error(y)`. */
    case T_NON:
      return 1;
    case T_ERR:
      return _eq(a.items[0], b.items[0]);
    case T_MAP: {
      if (a.argc != b.argc) return 0;
      for (int i = 0; i < a.argc; i++) {
        if (!_eq(a.items[2 * i], b.items[2 * i])) return 0;
        if (!_eq(a.items[2 * i + 1], b.items[2 * i + 1])) return 0;
      }
      return 1;
    }
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

/* BUG-110: la VM rechaza los desplazamientos fuera de 0-63, pero en C
 * `x << 64` es comportamiento indefinido: el binario nativo devolvia basura
 * (o 0, o el propio operando) mientras la VM daba un error claro. Se valida
 * igual que la VM para que ambos backends coincidan. */
static int64_t _shift_amt(int64_t y) {
  if (y < 0 || y > 63) {
    char _m[96];
    snprintf(_m, sizeof(_m), "Desplazamiento %lld fuera de rango (0-63)", (long long)y);
    _rt_error3(_m);
  }
  return y;
}

static Val _arith(int op, Val a, Val b) {
  int isf = a.t == T_FLT || b.t == T_FLT;
  if (!isf) {
    int64_t x = a.i, y = b.i;
    switch (op) {
      case 1: return _v_int(x + y);
      case 3: return _v_int(x - y);
      case 4: return _v_int(x * y);
      case 5: if (!y) { _rt_error3("División por cero"); } return _v_int(x / y);
      case 6: if (!y) { _rt_error3("División por cero"); } return _v_int(x % y);
    }
    return _v_int(0);
  }
  double x = _asf(a), y = _asf(b);
  switch (op) {
    case 1: return _v_flt(x + y);
    case 3: return _v_flt(x - y);
    case 4: return _v_flt(x * y);
    /* BUG-089: la division decimal por cero daba `inf`/`nan` en el binario
       nativo mientras que la VM aborta con "Division por cero". El mismo
       programa terminaba bien compilado e imprimia inf, o fallaba con la VM. */
    case 5: if (y == 0.0) { _rt_error3("División por cero"); } return _v_flt(x / y);
    case 6: if (y == 0.0) { _rt_error3("División por cero"); } return _v_flt(fmod(x, y));
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
      case 5:  if (!y) { _rt_error3("División por cero"); } return (Val){.i = x / y, .t = T_INT};
      case 6:  if (!y) { _rt_error3("División por cero"); } return (Val){.i = x % y, .t = T_INT};
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
      case 17: return (Val){.i = x << _shift_amt(y), .t = T_INT};
      case 18: return (Val){.i = x >> _shift_amt(y), .t = T_INT};
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
    case 17: return _v_int((int64_t)_asf(a) << _shift_amt((int64_t)_asf(b)));
    case 18: return _v_int((int64_t)_asf(a) >> _shift_amt((int64_t)_asf(b)));
    case 19: return _v_int((int64_t)_asf(a) ^ (int64_t)_asf(b));
  }
  return _v_int(0);
}

static Val _neg(Val a) {
  if (a.t == T_FLT) return _v_flt(-a.f);
  return _v_int(-a.i);
}
static Val _not(Val a) { return _v_bool(!_truthy(a)); }
/* BUG-112: `_asf` convierte a `double`, y un `double` no puede representar
   todos los int64: `~9223372036854775807` daba 9223372036854775807 en el
   binario nativo (el valor se redondeaba al pasar por coma flotante) mientras
   la VM daba -9223372036854775808. El complemento a uno es una operación
   entera; no debe pasar por `double`. */
static Val _bnot(Val a) {
  int64_t x = (a.t == T_FLT) ? (int64_t)a.f : a.i;
  return _v_int(~x);
}

/* BUG-083: `_dcp` clonaba en PROFUNDIDAD en cada paso de argumento. Un bucle
   que acumulaba en una lista de structs (el patron `b = f(b, ...)`, habitual en
   la stdlib grafica) copiaba la lista entera en cada vuelta: coste O(n^2) en
   tiempo Y en memoria, porque ademas ninguna copia se liberaba nunca. Con 400
   elementos el binario ya gastaba 534 MB y con 800 lo mataba el OOM killer,
   mientras la VM ejecutaba lo mismo sin despeinarse.

   Ahora la copia es PEREZOSA (copy-on-write): `_dcp` comparte el buffer y lo
   marca como compartido; solo los dos puntos que mutan in situ (`_arr_set` y
   `_st_set`) materializan una copia privada antes de escribir. La semantica de
   valor se conserva: nadie observa una escritura ajena. */
static inline Val _cow_unshare(Val v);
static inline Val _dcp(Val v) {
  if (__builtin_expect(v.t <= T_BOL || v.t == T_STR || v.t == T_NON || v.t == T_VOD, 1)) return v;
  if (v.t == T_ARR || v.t == T_TUP || v.t == T_ENM || v.t == T_MAP || v.t == T_STT) {
    Val nv = v;
    nv.shared = 1;
    return nv;
  }

  return v;
}

/* BUG-083: materializa una copia privada del buffer si estaba compartido.
   Solo se llama desde los puntos que mutan `items` in situ. */
static inline Val _cow_unshare(Val v) {
  if (!v.shared || !v.items) { v.shared = 0; return v; }
  int n = (v.t == T_MAP || v.t == T_STT) ? v.argc * 2 : v.argc;
  Val* ni = (Val*)malloc(sizeof(Val) * (size_t)(n > 0 ? n : 1));
  for (int i = 0; i < n; i++) {
    ni[i] = v.items[i];
    /* Los hijos quedan referenciados por DOS padres (el original y esta
       copia), asi que pasan a estar compartidos ellos tambien. */
    if (ni[i].items) ni[i].shared = 1;
  }
  v.items = ni;
  v.shared = 0;
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
  a.shared = 0; /* BUG-083: buffer recien creado, es privado */
  return a;
}
/* BUG-041: indexar un texto (`s[0]`) reventaba con "Indice 0 fuera de rango
   (largo: 0)" en los binarios nativos, porque `_arr_get` sólo miraba `argc`,
   que en un texto vale 0. La VM devuelve el CARÁCTER en esa posición (no el
   byte), así que aquí recorremos la cadena respetando UTF-8. */
static Val _str_idx(Val a, int64_t ix) {
  const char* p = a.s ? a.s : "";
  int64_t n = 0;
  while (*p) {
    const unsigned char* q = (const unsigned char*)p;
    int len = 1;
    if ((*q & 0x80) != 0) {
      if ((*q & 0xE0) == 0xC0) len = 2;
      else if ((*q & 0xF0) == 0xE0) len = 3;
      else if ((*q & 0xF8) == 0xF0) len = 4;
    }
    if (n == ix) {
      char* b = (char*)malloc((size_t)len + 1);
      for (int i = 0; i < len; i++) b[i] = p[i];
      b[len] = 0;
      return _v_str(b);
    }
    p += len;
    n++;
  }
  { char _m[128]; snprintf(_m, sizeof(_m), "Índice %lld fuera de rango (largo: %lld)", (long long)ix, (long long)n); _rt_error3(_m); }
  return _v_void();
}

static Val _arr_get(Val a, int64_t ix) {
  if (a.t == T_STR) return _str_idx(a, ix);
  if (a.t == T_MAP) {
    /* BUG-041: `m[k]` con clave entera sobre un mapa debe buscar la clave,
       no tratar el mapa como un array posicional. */
    for (int i = 0; i < a.argc; i++)
      if (_eq(a.items[2 * i], _v_int(ix))) return a.items[2 * i + 1];
    return _v_void();
  }
  if (ix < 0 || ix >= a.argc) {
    { char _m[128]; snprintf(_m, sizeof(_m), "Índice %lld fuera de rango (largo: %d)", (long long)ix, a.argc); _rt_error3(_m); }
  }
  {
    /* BUG-083: si el contenedor esta compartido, el hijo tambien lo esta:
       mutarlo in situ escribiria en el buffer del original. */
    Val _c = a.items[ix];
    if (a.shared && _c.items) _c.shared = 1;
    return _c;
  }
}
static Val _arr_set(Val a, int64_t ix, Val x) {
  if (ix < 0 || ix >= a.argc) {
    { char _m[128]; snprintf(_m, sizeof(_m), "Índice %lld fuera de rango (largo: %d)", (long long)ix, a.argc); _rt_error3(_m); }
  }
  a = _cow_unshare(a); /* BUG-083 */
  a.items[ix] = x;
  return a;
}
/* BUG-035: `largo(texto)` debe contar CARACTERES, como hace la VM
   (`chars().count()`), no bytes. `strlen` daba 13 donde la VM decía 7 para
   "áéíóú ñ", así que el mismo programa producía resultados distintos según el
   backend. Contamos los bytes que no son continuación UTF-8 (10xxxxxx). */
static int64_t _utf8_len(const char* s) {
  if (!s) return 0;
  int64_t n = 0;
  for (const unsigned char* p = (const unsigned char*)s; *p; p++) {
    if ((*p & 0xC0) != 0x80) n++;
  }
  return n;
}
/* BUG-048: `s.largo()` sobre un texto devolvía 0 (los textos no usan `argc`),
   así que un `mientras i < s.largo()` no entraba nunca y la función devolvía
   "" sin error. La forma `largo(s)` sí funcionaba: dos caminos, un solo
   resultado esperado. */
static Val _arr_len(Val a) {
  if (a.t == T_STR) return _v_int(_utf8_len(a.s));
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

/* BUG-037: soporte de `__frame_param`, que implementa el write-back de
   `prestado mut` (BUG-020). La VM guarda el marco recién retornado en
   `last_frame`; aquí hacemos lo mismo copiando los parámetros del callee a
   `_fp` JUSTO al volver de la llamada, antes de que el llamador restaure sus
   propias variables (si comparten nombre, la restauración los pisaría). */
#define MAX_FP 32
static Val _fp[MAX_FP];
static int _fpc = 0;
static Val _frame_param(int64_t i) {
  if (i >= 0 && i < _fpc) return _fp[i];
  return _v_void();
}

/* BUG-039: `a_entero_seguro` / `a_decimal_seguro` estaban declaradas como
   builtins ensombrecibles pero NO implementadas en el backend C: la llamada
   caía en el camino de función desconocida, devolvía void y el `elegir` sobre
   el `resultado` no casaba con ningún caso, así que no se imprimía nada.
   Se replica la semántica de la VM, incluido el texto del error. */
static Val _b_a_entero_seguro(Val v) {
  double d;
  if (!_parse_f64(v, &d)) {
    char* b = (char*)malloc(512);
    snprintf(b, 512, "no se puede convertir '%s' a entero", _fmt(v));
    return _res(_v_str(b), 0);
  }
  return _res(_v_int((int64_t)d), 1);
}
static Val _b_a_decimal_seguro(Val v) {
  double d;
  if (!_parse_f64(v, &d)) {
    char* b = (char*)malloc(512);
    snprintf(b, 512, "no se puede convertir '%s' a decimal", _fmt(v));
    return _res(_v_str(b), 0);
  }
  return _res(_v_flt(d), 1);
}
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
/* BUG-036: el patrón de `elegir` sobre enums se compila a llamadas a
   `__enum_variante` / `__enum_campo` / `__enum_aridad`. Existían en la VM pero
   NO en el backend C, donde caían en el camino de "función desconocida" y
   devolvían void: ningún caso casaba y el `elegir` entero no imprimía nada.
   Se replica aquí la semántica exacta de la VM, incluidos `algun/ninguno` y
   `exito/error`, que usan el mismo protocolo. */
static const char* _enum_variante(Val v) {
  switch (v.t) {
    case T_ENM: return v.vr ? v.vr : "";
    case T_SOM: return "algun";
    case T_NON: return "ninguno";
    case T_OK:  return "exito";
    case T_ERR: return "error";
    default:    return "";
  }
}
static Val _enum_campo(Val v, int64_t i) {
  if (v.t == T_ENM) {
    if (i >= 0 && i < v.argc) return v.items[i];
    return _v_void();
  }
  if ((v.t == T_SOM || v.t == T_OK || v.t == T_ERR) && i == 0) {
    return v.argc > 0 ? v.items[0] : _v_void();
  }
  return _v_void();
}
static int64_t _enum_aridad(Val v) {
  if (v.t == T_ENM) return v.argc;
  if (v.t == T_SOM || v.t == T_OK || v.t == T_ERR) return 1;
  return 0;
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
    if (!strcmp(s.items[2 * i].s, f)) {
      Val _c = s.items[2 * i + 1];
      if (s.shared && _c.items) _c.shared = 1; /* BUG-083 */
      return _c;
    }
  }
  { char _m[160]; snprintf(_m, sizeof(_m), "Campo '%s' no encontrado en struct", f); _rt_error3(_m); }
  return _v_int(0);
}
static Val _st_set(Val s, const char* f, Val x) {
  for (int i = 0; i < s.argc; i++) {
    if (!strcmp(s.items[2 * i].s, f)) {
      s = _cow_unshare(s); /* BUG-083 */
      s.items[2 * i + 1] = x;
      return s;
    }
  }
  { char _m[160]; snprintf(_m, sizeof(_m), "Campo '%s' no encontrado en struct", f); _rt_error3(_m); }
  return s;
}

static char* _fmt(Val v) {
  char* b = (char*)malloc(8192);
  if (!b) return (char*)"";
  switch (v.t) {
    case T_INT:
      snprintf(b, 8192, "%lld", (long long)v.i);
      break;
    case T_FLT: {
      double d = v.f;
      /* BUG-113: `isinf` estaba contemplado pero `isnan` no, asi que el NaN
         caia al `%g` de abajo y salia como "-nan" (o "nan" segun el signo del
         bit), mientras la VM imprime "NaN". Mismo calculo, dos textos. */
      if (isnan(d)) { snprintf(b, 8192, "NaN"); break; }
      if (isinf(d)) { snprintf(b, 8192, d < 0 ? "-inf" : "inf"); break; }
      /* BUG-115: el guard `fabs(d) < 1e16` mandaba los decimales grandes al
         `%g` de abajo, que cambia a NOTACION CIENTIFICA: `1000000000000000.0 *
         1000` se imprimia como "1e+18" en el binario nativo y como
         "1000000000000000000" en la VM. Igual por abajo: 0.000001 salia como
         "1e-06". El `Display` de Rust para f64 nunca usa notacion cientifica,
         asi que aqui tampoco. El limite real es el rango en que un double
         representa enteros de forma exacta (2^63); mas alla no cabe en
         `long long` y hay que formatear con `%.1f`, que tampoco la usa. */
      /* El limite es ESTRICTO: 9223372036854775807.0 se redondea a 2^63 al
         guardarse en un double, y ahi `(int64_t)d` vuelve a ser UB. Se usa el
         mayor double que cabe con seguridad en int64. */
      if (fabs(d) < 9223372036854775296.0 && d == (double)(int64_t)d) {
        snprintf(b, 8192, "%lld", (long long)d);
      } else if (d == floor(d) && isfinite(d)) {
        snprintf(b, 8192, "%.1f", d);
        /* "%.1f" deja un ".0" final que la VM no imprime para enteros. */
        { size_t _n = strlen(b);
          if (_n > 2 && b[_n - 2] == '.' && b[_n - 1] == '0') b[_n - 2] = '\0'; }
      } else {
        int _p; char _t[512];
        for (_p = 1; _p <= 17; _p++) {
          snprintf(_t, sizeof _t, "%.*g", _p, d);
          if (strtod(_t, NULL) == d) break;
        }
        if (_p > 17) snprintf(_t, sizeof _t, "%.17g", d);
        /* Si `%g` eligio notacion cientifica, reformatear en decimal plano con
           los digitos significativos que hagan falta para no perder precision. */
        if (strchr(_t, 'e') || strchr(_t, 'E')) {
          int _q;
          for (_q = 1; _q <= 30; _q++) {
            snprintf(b, 8192, "%.*f", _q, d);
            if (strtod(b, NULL) == d) break;
          }
          if (_q > 30) snprintf(b, 8192, "%.17f", d);
        } else {
          snprintf(b, 8192, "%s", _t);
        }
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
      /* BUG-159: paridad con la VM; `__tipo_de` ya decia "nulo". */
      snprintf(b, 8192, "nulo");
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
  /* BUG-040: siempre se añadía al final, sin mirar si la clave ya estaba.
     Volver a poner una clave existente dejaba DOS entradas y `_map_get`
     devolvía la vieja (recorre desde el principio), así que el valor nunca
     parecía actualizarse y `_map_longitud` crecía de más. La VM usa un mapa
     persistente real, donde `insert` reemplaza. */
  for (int i = 0; i < n; i++) {
    if (_eq(m.items[2 * i], k)) {
      Val* ri = (Val*)malloc(sizeof(Val) * (size_t)(n > 0 ? n : 1) * 2);
      for (int j = 0; j < n * 2; j++) ri[j] = m.items[j];
      ri[2 * i + 1] = x;
      nv.items = ri; nv.argc = n;
      return nv;
    }
  }
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
/* BUG-088: el orden de las claves difería entre la VM (orden de hash) y el
   binario nativo (orden de inserción), asi que imprimir __map_claves daba
   resultados distintos segun el backend. Ninguno de los dos ordenes era
   significativo, asi que ambos ordenan ahora de forma estable: numeros por
   valor y el resto por su representacion textual. */
static int _key_cmp(const void* pa, const void* pb) {
  const Val* a = (const Val*)pa; const Val* b = (const Val*)pb;
  int na = (a->t == T_INT || a->t == T_FLT), nb = (b->t == T_INT || b->t == T_FLT);
  if (na && nb) {
    double da = (a->t == T_INT) ? (double)a->i : a->f;
    double db = (b->t == T_INT) ? (double)b->i : b->f;
    return (da < db) ? -1 : (da > db) ? 1 : 0;
  }
  if (na != nb) return na ? -1 : 1;
  return strcmp(_fmt(*a), _fmt(*b));
}
static Val _map_keys(Val m) {
  Val* ns = (Val*)malloc(sizeof(Val) * (m.argc + 1));
  for (int i = 0; i < m.argc; i++) ns[i] = m.items[2 * i];
  if (m.argc > 1) qsort(ns, (size_t)m.argc, sizeof(Val), _key_cmp);
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

#if !defined(_WIN32) && !defined(__APPLE__)
/* BUG-080: la VM usa sintaxis tipo Perl (\d, \w, \s, \D, \W, \S) pero POSIX
   REG_EXTENDED no la conoce: regcomp fallaba y _regex_m devolvia 0, de modo que
   el binario nativo decia "false" donde la VM decia "true". Se traducen esas
   clases a sus equivalentes POSIX antes de compilar el patron. */
static char* _regex_posix(const char* pat) {
  size_t n = strlen(pat);
  size_t cap = n * 12 + 16;
  char* out = (char*)malloc(cap);
  if (!out) return NULL;
  size_t o = 0;
  for (size_t i = 0; i < n; i++) {
    if (pat[i] == '\\' && i + 1 < n) {
      const char* rep = NULL;
      switch (pat[i + 1]) {
        case 'd': rep = "[0-9]"; break;
        case 'D': rep = "[^0-9]"; break;
        case 'w': rep = "[a-zA-Z0-9_]"; break;
        case 'W': rep = "[^a-zA-Z0-9_]"; break;
        case 's': rep = "[ \t\n\r\f\v]"; break;
        case 'S': rep = "[^ \t\n\r\f\v]"; break;
        default: break;
      }
      if (rep) {
        size_t rl = strlen(rep);
        memcpy(out + o, rep, rl);
        o += rl;
        i++;
        continue;
      }
      /* escape que POSIX si entiende: se copia tal cual */
      out[o++] = pat[i];
      out[o++] = pat[i + 1];
      i++;
      continue;
    }
    out[o++] = pat[i];
  }
  out[o] = 0;
  return out;
}
static int _regex_m(const char* pat, const char* s) {
  regex_t re;
  char* tp = _regex_posix(pat);
  const char* use = tp ? tp : pat;
  if (regcomp(&re, use, REG_EXTENDED) != 0) { if (tp) free(tp); return 0; }
  if (tp) free(tp);
  int r = regexec(&re, s, 0, NULL, 0);
  regfree(&re);
  return r == 0;
}
static char* _regex_rep(const char* pat, const char* s, const char* rep) {
  regex_t re;
  char* tp = _regex_posix(pat);
  const char* use = tp ? tp : pat;
  if (regcomp(&re, use, REG_EXTENDED) != 0) { if (tp) free(tp); return (char*)s; }
  if (tp) free(tp);
  size_t sn = strlen(s), rn = strlen(rep);
  /* Peor caso: una sustitucion vacia entre cada caracter, mas una al final. */
  size_t cap = sn + (rn + 1) * (sn + 2) + 16;
  char* out = (char*)malloc(cap);
  if (!out) { regfree(&re); return (char*)s; }
  size_t oi = 0, i = 0;
  /* BUG-167: el bucle antiguo hacia `p += 1` ante una coincidencia vacia sin
     comprobar si ya estaba en el terminador, asi que se salia de la cadena y
     seguia copiando memoria ajena: `__regex_reemplazar("[a-z]?|a","x_y","#")`
     terminaba en SIGSEGV. Ahora se recorre por indice con `i <= sn` y se para
     en el limite. `ultimo_fin` replica la regla de la VM: una coincidencia
     vacia justo donde acabo la anterior no cuenta, de modo que "a?" sobre
     "bab" da "#b#b#" y no "#b##b#". */
  size_t ultimo_fin = (size_t)-1;
  regmatch_t m;
  while (i <= sn) {
    int flags = (i == 0) ? 0 : REG_NOTBOL;
    if (regexec(&re, s + i, 1, &m, flags) != 0) break;
    size_t ini_m = i + (size_t)m.rm_so;
    size_t fin_m = i + (size_t)m.rm_eo;
    if (ini_m == fin_m && ini_m == ultimo_fin) {
      /* Coincidencia vacia pegada a la anterior: se ignora y se avanza. */
      if (i >= sn) break;
      out[oi++] = s[i];
      i++;
      continue;
    }
    memcpy(out + oi, s + i, ini_m - i);
    oi += ini_m - i;
    memcpy(out + oi, rep, rn);
    oi += rn;
    ultimo_fin = fin_m;
    if (fin_m == ini_m) {
      if (ini_m >= sn) { i = sn + 1; break; }
      out[oi++] = s[ini_m];
      i = ini_m + 1;
    } else {
      i = fin_m;
    }
  }
  if (i <= sn) {
    memcpy(out + oi, s + i, sn - i);
    oi += sn - i;
  }
  out[oi] = 0;
  regfree(&re);
  return out;
}
#else
/* Rama Windows/macOS: <regex.h> POSIX no esta disponible.

   BUG-166: aqui habia un stub que devolvia SIEMPRE 0. En Windows y macOS
   `__regex_coincide` respondia "false" a cualquier patron mientras la VM
   respondia "true": el mismo BUG-080 que se arreglo para Linux, pero vivo en
   las otras dos plataformas. Un binario que contesta que no a todo es peor que
   uno que falla, porque parece que funciona.

   POSIX <regex.h> no esta disponible ahi, asi que se implementa un motor
   propio por backtracking que cubre lo que la VM acepta y este runtime usa:
   literales, `.`, clases `[...]` con rangos y negacion, las clases Perl
   \d \D \w \W \s \S, los cuantificadores `*` `+` `?`, anclas `^` `$`,
   alternacion `|` y grupos `(...)`. Sin capturas: `_regex_m` solo necesita
   saber si casa, y `_regex_rep` sustituye la porcion encontrada. */
typedef struct { const char* p; const char* pend; } _rxpat;

static int _rx_alt(const char* p, const char* pend, const char* s,
                   const char* sbeg, const char* send, const char** mend);

/* Devuelve el final del elemento que empieza en `p` (atomo + cuantificador). */
static const char* _rx_atom_end(const char* p, const char* pend) {
  if (p >= pend) return p;
  if (*p == '\\' && p + 1 < pend) {
    p += 2;
  } else if (*p == '[') {
    p++;
    if (p < pend && *p == '^') p++;
    if (p < pend && *p == ']') p++;
    while (p < pend && *p != ']') p++;
    if (p < pend) p++;
  } else if (*p == '(') {
    int d = 1;
    p++;
    while (p < pend && d > 0) {
      if (*p == '\\' && p + 1 < pend) { p += 2; continue; }
      if (*p == '(') d++;
      else if (*p == ')') d--;
      p++;
    }
  } else {
    p++;
  }
  return p;
}

/* Cuantificador `{n}`, `{n,}` o `{n,m}`. `q` apunta a la llave de apertura.
   Devuelve 0 si no es un cuantificador valido: entonces la llave se trata como
   un caracter literal, que es lo que hace la VM. */
static int _rx_llaves(const char* q, const char* pend, int* min, int* max,
                      const char** after) {
  if (q >= pend || *q != '{') return 0;
  const char* r = q + 1;
  int lo = 0, hi = -1, ndig = 0;
  while (r < pend && *r >= '0' && *r <= '9') { lo = lo * 10 + (*r - '0'); r++; ndig++; }
  if (ndig == 0) return 0;
  if (r < pend && *r == '}') { hi = lo; }
  else if (r < pend && *r == ',') {
    r++;
    int mdig = 0, v = 0;
    while (r < pend && *r >= '0' && *r <= '9') { v = v * 10 + (*r - '0'); r++; mdig++; }
    if (r >= pend || *r != '}') return 0;
    hi = mdig ? v : -1;
  } else {
    return 0;
  }
  if (hi >= 0 && hi < lo) return 0;
  *min = lo;
  *max = hi;
  *after = r + 1;
  return 1;
}

static int _rx_cls(char c, char k) {
  switch (k) {
    case 'd': return c >= '0' && c <= '9';
    case 'D': return !(c >= '0' && c <= '9');
    case 'w': return (c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z') ||
                     (c >= '0' && c <= '9') || c == '_';
    case 'W': return !((c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z') ||
                       (c >= '0' && c <= '9') || c == '_');
    case 's': return c == ' ' || c == '\t' || c == '\n' || c == '\r' ||
                     c == '\f' || c == '\v';
    case 'S': return !(c == ' ' || c == '\t' || c == '\n' || c == '\r' ||
                       c == '\f' || c == '\v');
    default: return c == k;
  }
}

/* ¿Casa el atomo simple [p,ae) con el caracter *s? (no vale para grupos) */
static int _rx_one(const char* p, const char* ae, char c) {
  if (*p == '.') return 1;
  if (*p == '\\' && p + 1 < ae) return _rx_cls(c, p[1]);
  if (*p == '[') {
    const char* q = p + 1;
    int neg = 0, hit = 0;
    if (q < ae && *q == '^') { neg = 1; q++; }
    const char* cend = ae - 1; /* apunta a ']' */
    while (q < cend) {
      if (*q == '\\' && q + 1 < cend) {
        if (_rx_cls(c, q[1])) hit = 1;
        q += 2;
        continue;
      }
      if (q + 2 < cend && q[1] == '-') {
        if ((unsigned char)c >= (unsigned char)q[0] &&
            (unsigned char)c <= (unsigned char)q[2]) hit = 1;
        q += 3;
        continue;
      }
      if (*q == c) hit = 1;
      q++;
    }
    return neg ? !hit : hit;
  }
  return *p == c;
}

/* Secuencia (sin `|` de primer nivel). */
static int _rx_seq(const char* p, const char* pend, const char* s,
                   const char* sbeg, const char* send, const char** mend) {
  if (p >= pend) { if (mend) *mend = s; return 1; }

  if (*p == '^') {
    if (s != sbeg) return 0;
    return _rx_seq(p + 1, pend, s, sbeg, send, mend);
  }
  if (*p == '$' && p + 1 == pend) {
    if (s != send) return 0;
    if (mend) *mend = s;
    return 1;
  }

  const char* ae = _rx_atom_end(p, pend);
  char q = (ae < pend) ? *ae : 0;
  const char* rest = (q == '*' || q == '+' || q == '?') ? ae + 1 : ae;
  int grupo = (*p == '(');
  int min = -1, max = -1;
  if (q == '*') { min = 0; max = -1; }
  else if (q == '+') { min = 1; max = -1; }
  else if (q == '?') { min = 0; max = 1; }
  else if (q == '{') {
    const char* tras = NULL;
    if (_rx_llaves(ae, pend, &min, &max, &tras)) rest = tras;
    else min = -1;
  }

  if (min >= 0) {
    /* Voraz con retroceso: se prueban las repeticiones de mas a menos. */
    const char* puntos[4096];
    int n = 0;
    const char* cur = s;
    puntos[n++] = cur;
    while ((max < 0 || n - 1 < max) && n < 4096) {
      const char* nx = NULL;
      if (grupo) {
        if (!_rx_alt(p + 1, ae - 1, cur, sbeg, send, &nx)) break;
        if (nx == cur) break; /* grupo vacio: evita bucle infinito */
      } else {
        if (cur >= send || !_rx_one(p, ae, *cur)) break;
        nx = cur + 1;
      }
      cur = nx;
      puntos[n++] = cur;
    }
    while (n - 1 >= min) {
      if (_rx_seq(rest, pend, puntos[n - 1], sbeg, send, mend)) return 1;
      n--;
    }
    return 0;
  }

  if (grupo) {
    /* Sin cuantificador: probar cada final posible del grupo. */
    const char* nx = NULL;
    if (!_rx_alt(p + 1, ae - 1, s, sbeg, send, &nx)) return 0;
    return _rx_seq(rest, pend, nx, sbeg, send, mend);
  }

  if (s >= send || !_rx_one(p, ae, *s)) return 0;
  return _rx_seq(rest, pend, s + 1, sbeg, send, mend);
}

/* Alternacion de primer nivel: divide por `|` fuera de grupos y clases. */
static int _rx_alt(const char* p, const char* pend, const char* s,
                   const char* sbeg, const char* send, const char** mend) {
  const char* ini = p;
  const char* q = p;
  int d = 0;
  while (q < pend) {
    if (*q == '\\' && q + 1 < pend) { q += 2; continue; }
    if (*q == '[') { q = _rx_atom_end(q, pend); continue; }
    if (*q == '(') d++;
    else if (*q == ')') d--;
    else if (*q == '|' && d == 0) {
      if (_rx_seq(ini, q, s, sbeg, send, mend)) return 1;
      ini = q + 1;
    }
    q++;
  }
  return _rx_seq(ini, pend, s, sbeg, send, mend);
}

static int _regex_m(const char* pat, const char* s) {
  size_t pn = strlen(pat), sn = strlen(s);
  const char* pend = pat + pn;
  const char* send = s + sn;
  for (const char* st = s; st <= send; st++) {
    if (_rx_alt(pat, pend, st, s, send, NULL)) return 1;
    if (pn > 0 && pat[0] == '^') break; /* anclado: solo desde el inicio */
  }
  return 0;
}

static char* _regex_rep(const char* pat, const char* s, const char* rep) {
  size_t pn = strlen(pat), sn = strlen(s), rn = strlen(rep);
  const char* pend = pat + pn;
  const char* send = s + sn;
  size_t cap = sn + (rn + 1) * (sn + 2) + 16;
  char* out = (char*)malloc(cap);
  if (!out) return (char*)s;
  size_t o = 0;
  const char* cur = s;
  /* BUG-167: misma regla que la rama POSIX. Una coincidencia vacia pegada al
     final de la anterior no se sustituye, para que ambas ramas y la VM den
     exactamente el mismo texto. */
  const char* ultimo_fin = NULL;
  while (cur <= send) {
    const char* fin = NULL;
    if (_rx_alt(pat, pend, cur, s, send, &fin) && fin) {
      if (fin == cur && cur == ultimo_fin) {
        if (cur >= send) break;
        out[o++] = *cur;
        cur++;
        continue;
      }
      memcpy(out + o, rep, rn);
      o += rn;
      ultimo_fin = fin;
      if (fin == cur) { /* coincidencia vacia: avanza uno para no colgarse */
        if (cur >= send) { cur = send + 1; break; }
        out[o++] = *cur;
        cur++;
      } else {
        cur = fin;
      }
      continue;
    }
    if (cur < send) out[o++] = *cur;
    cur++;
  }
  out[o] = 0;
  return out;
}
#endif

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
/* BUG-043: sólo cambiaba de caja el ASCII, así que `mayusculas("Lúmen")`
   devolvía "LúMEN" (la `ú` intacta) mientras la VM daba "LÚMEN". Se añaden
   las vocales acentuadas y la eñe del bloque Latin-1, que en UTF-8 ocupan dos
   bytes (0xC3 0xA0..0xBE); la conversión allí es restar/sumar 0x20 al segundo
   byte, salvo 0xC3 0xB7 (÷), que no es una letra. */
static char* _case_str(const char* s, int up) {
  size_t n = strlen(s);
  char* m = (char*)malloc(n + 1);
  size_t i = 0;
  while (i < n) {
    unsigned char c = (unsigned char)s[i];
    if (c == 0xC3 && i + 1 < n) {
      unsigned char d = (unsigned char)s[i + 1];
      if (up && d >= 0xA0 && d <= 0xBE && d != 0xB7) d = (unsigned char)(d - 0x20);
      else if (!up && d >= 0x80 && d <= 0x9E && d != 0x97) d = (unsigned char)(d + 0x20);
      m[i] = (char)c;
      m[i + 1] = (char)d;
      i += 2;
      continue;
    }
    char e = s[i];
    if (up && e >= 'a' && e <= 'z') e = e - 32;
    else if (!up && e >= 'A' && e <= 'Z') e = e + 32;
    m[i] = e;
    i++;
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
static Val _to_chars(const char* s); /* BUG-122: se usa antes de definirse */
static Val _str_split(const char* s, const char* delim) {
  if (!s) return _arrn(NULL, 0);
  if (!delim || !delim[0]) {
    /* BUG-122: misma familia que BUG-087. Con separador vacio se partia por
       BYTES, asi que "\xc3\xb1o\xc3\xb1o" daba 6 trozos rotos en el binario
       nativo y 4 caracteres en la VM. `_to_chars` ya decodifica UTF-8. */
    return _to_chars(s);
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
/* BUG-082: `_sub` y `_to_chars` indexaban por BYTES mientras la VM (y `largo`,
   que ya usaba _utf8_len) cuentan CARACTERES. Con acentos el binario nativo
   cortaba en distinto sitio que la VM y podia partir un caracter por la mitad,
   emitiendo UTF-8 invalido ("áé\xef\xbf\xbd"). Se convierte el indice de
   caracter a offset de byte antes de cortar. */
static size_t _utf8_off(const char* s, int64_t chars) {
  size_t i = 0;
  int64_t c = 0;
  while (s[i] && c < chars) {
    unsigned char b = (unsigned char)s[i];
    i += (b < 0x80) ? 1 : ((b >> 5) == 0x6) ? 2 : ((b >> 4) == 0xE) ? 3 : ((b >> 3) == 0x1E) ? 4 : 1;
    c++;
  }
  return i;
}
/* BUG-120: los indices negativos NO cuentan desde el final. La VM convierte el
   entero a `usize` (un negativo pasa a ser un numero enorme) y luego lo recorta
   a la longitud, asi que un inicio negativo da SIEMPRE cadena vacia. El C hacia
   `if (st < 0) st = 0`, o sea tomaba desde el principio: `__str_slice("hola",
   -2, -1)` daba "" en la VM y "hola" ya compilado. Se replica la VM.
   El unico negativo con significado es `en == -1`, que la VM trata como "hasta
   el final" de forma explicita. */
static char* _sub(const char* s, int64_t st, int64_t en) {
  int64_t n = _utf8_len(s);
  if (st < 0) st = n; if (st > n) st = n;
  if (en == -1) en = n; else if (en < 0) en = n; if (en > n) en = n;
  if (en < st) en = st;
  size_t bs = _utf8_off(s, st);
  size_t be = _utf8_off(s, en);
  char* m = (char*)malloc(be - bs + 1);
  memcpy(m, s + bs, be - bs); m[be - bs] = 0;
  return m;
}
static Val _to_chars(const char* s) {
  int64_t n = _utf8_len(s);
  Val* xs = (Val*)malloc(sizeof(Val) * (size_t)(n + 1));
  size_t off = 0;
  for (int64_t i = 0; i < n; i++) {
    unsigned char b = (unsigned char)s[off];
    size_t w = (b < 0x80) ? 1 : ((b >> 5) == 0x6) ? 2 : ((b >> 4) == 0xE) ? 3 : ((b >> 3) == 0x1E) ? 4 : 1;
    char c[5]; memcpy(c, s + off, w); c[w] = 0;
    xs[i] = _v_str(c);
    off += w;
  }
  Val v = _arrn(xs, (size_t)n); free(xs);
  return v;
}
/* BUG-087: devolvia un byte por elemento, asi que "an~b" daba
   [97,195,177,98] en el binario nativo y [97,241,98] en la VM. Ahora decodifica
   UTF-8 y devuelve puntos de codigo, igual que la VM. */
static Val _str_codes(const char* s) {
  size_t nb = strlen(s);
  Val* xs = (Val*)malloc(sizeof(Val) * (nb + 1));
  size_t i = 0, k = 0;
  while (i < nb) {
    unsigned char b = (unsigned char)s[i];
    uint32_t cp; size_t w;
    if (b < 0x80) { cp = b; w = 1; }
    else if ((b >> 5) == 0x6 && i + 1 < nb) { cp = ((uint32_t)(b & 0x1F) << 6) | ((unsigned char)s[i+1] & 0x3F); w = 2; }
    else if ((b >> 4) == 0xE && i + 2 < nb) { cp = ((uint32_t)(b & 0x0F) << 12) | (((unsigned char)s[i+1] & 0x3F) << 6) | ((unsigned char)s[i+2] & 0x3F); w = 3; }
    else if ((b >> 3) == 0x1E && i + 3 < nb) { cp = ((uint32_t)(b & 0x07) << 18) | (((unsigned char)s[i+1] & 0x3F) << 12) | (((unsigned char)s[i+2] & 0x3F) << 6) | ((unsigned char)s[i+3] & 0x3F); w = 4; }
    else { cp = b; w = 1; }
    xs[k++] = _v_int((int64_t)cp);
    i += w;
  }
  Val v = _arrn(xs, k); free(xs);
  return v;
}
/* BUG-121: con patron vacio el C devolvia el texto intacto, pero `str::replace`
   de Rust (que es lo que hace la VM) inserta el reemplazo en cada frontera de
   caracter, incluidos los extremos: "aaa".replace("", "X") == "XaXaXaX". */
static char* _replace_vacio(const char* s, const char* to) {
  size_t n = strlen(s), tl = strlen(to);
  size_t nc = _utf8_len(s);
  char* out = (char*)malloc(n + tl * (nc + 1) + 1);
  size_t ln = 0, i = 0;
  memcpy(out + ln, to, tl); ln += tl;
  while (i < n) {
    unsigned char b = (unsigned char)s[i];
    size_t w = (b < 0x80) ? 1 : ((b >> 5) == 0x6) ? 2 : ((b >> 4) == 0xE) ? 3 : ((b >> 3) == 0x1E) ? 4 : 1;
    if (i + w > n) w = n - i;
    memcpy(out + ln, s + i, w); ln += w; i += w;
    memcpy(out + ln, to, tl); ln += tl;
  }
  out[ln] = 0;
  return out;
}
static char* _replace(const char* s, const char* from, const char* to) {
  if (!from || !from[0]) { return _replace_vacio(s, to); }
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
  _coro_stack_init();
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
  /* BUG-081: cada corrutina corre en su propio hilo y necesita inicializar SU
     pila de operandos y SU base de pila; antes heredaba los globales del hilo
     principal y el control de profundidad daba "pila agotada" al instante. */
  _coro_stack_init();
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