typedef signed char		int8_t;
typedef short int		int16_t;
typedef int			int32_t;
typedef long long int		int64_t;

typedef unsigned char		uint8_t;
typedef unsigned short int	uint16_t;
typedef unsigned int		uint32_t;
typedef unsigned long long int	uint64_t;

typedef int			int_fast16_t;
typedef int			int_fast32_t;

typedef unsigned int		uint_fast16_t;
typedef unsigned int		uint_fast32_t;

//typedef int			intptr_t;
//typedef unsigned int		uintptr_t;

#define __INT64_C(c)   c ## LL
#define __UINT64_C(c)  c ## ULL

#define __PRI64_RANK   "ll"
#define __PRIFAST_RANK ""
#define __PRIPTR_RANK  ""
typedef int8_t		int_least8_t;
typedef int16_t		int_least16_t;
typedef int32_t		int_least32_t;
typedef int64_t		int_least64_t;

typedef uint8_t		uint_least8_t;
typedef uint16_t	uint_least16_t;
typedef uint32_t	uint_least32_t;
typedef uint64_t	uint_least64_t;

typedef int8_t		int_fast8_t;
typedef int64_t		int_fast64_t;

typedef uint8_t		uint_fast8_t;
typedef uint64_t	uint_fast64_t;

typedef int64_t		intmax_t;
typedef uint64_t	uintmax_t;
