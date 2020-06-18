#lang rosette/safe

(provide
  (struct-out domain)
  make-concrete-domain
  all-constraints-satisfied?
  find-violated-tank-constraints
  find-violated-restrictions)

(require
  "core.rkt"
  "lift.rkt"
  (only-in racket error local-require)
  rosette/lib/match)

(module+ test
  (require
    (only-in racket for)
    (submod "core/tank.rkt" test)
    rackunit
    "test.rkt")

  ; make a fish for testing, 
  (define (make-simple-species [typ 't] . args)
    (define (not-exists? pred) (not (ormap pred args)))
    (define id (fresh-symbol))
    (define extra-args
      (filter
        (λ (x) x)
        (list
          (and (not-exists? size?) (make-size 1))
          (and (not-exists? environment?) (environment warm-water 50))
          (and (not-exists? diet?) (food 'x 1)))))
    (apply make-fish `(,id ,typ ,@args ,@extra-args)))
    
    (define (make-animal spc)
      (animal (fresh-integer) spc)))

(struct domain
  (in-tank?
   same-tank?
   tanks
   animals)
  #:transparent)

; (listof (pairof tank? (listof animal?))) -> domain?
(define (make-concrete-domain tank/animal-groups)
  (local-require (only-in racket hash-ref make-hash))

  (define animal->tank
    (let ([hm
            (make-hash
              (append-map
                (λ (pr)
                  (define tnk (tank-id (car pr)))
                  (map (λ (a) (cons (animal-id a) tnk)) (cdr pr)))
                tank/animal-groups))])
      (λ (ani) (hash-ref hm (animal-id ani)))))

  (domain
    (λ (tnk ani) (equal? (tank-id tnk) (animal->tank ani)))
    (λ (a1 a2) (equal? (animal->tank a1) (animal->tank a2)))
    (map car tank/animal-groups)
    (append-map cdr tank/animal-groups)))

(define (forall-in-tank f dom tnk)
  (landmap
    (λ (a)
      (=> ((domain-in-tank? dom) tnk a)
          (f a)))
    (domain-animals dom)))

(define (sum-in-tank f dom tnk)
  (sum
    (λ (a)
      (if ((domain-in-tank? dom) tnk a)
        (f a)
        0))
    (domain-animals dom)))

(define (same-species? x y)
  (equal? (animal-species-id x) (animal-species-id y)))

(module+ test
  (let ()
    (define s1 (make-simple-species))
    (define s2 (make-simple-species))
    (test-true "same-species? #t for same"
      (same-species? (animal 1 s1) (animal 2 s1)))
    (test-false "same-species? #f for different"
      (same-species? (animal 1 s1) (animal 2 s2)))))

(define (same-type? x y)
  (equal? (animal-type x) (animal-type y)))

(module+ test
  (let ()
    (define s1 (make-simple-species 'typ1))
    (define s2 (make-simple-species 'typ1))
    (define s3 (make-simple-species 'typ2))
    (test-true "same-type? #t for same species"
      (same-type? (animal 1 s1) (animal 2 s1)))
    (test-true "same-type? #t for same type diff species"
      (same-type? (animal 1 s1) (animal 2 s2)))
    (test-false "same-type? #f for different types"
      (same-type? (animal 1 s1) (animal 2 s3)))))

(define (count-in-same-tank f dom animl)
  (count
    (λ (other)
      (&& ((domain-same-tank? dom) animl other)
          (f other)))
    (domain-animals dom)))

(define (forall-in-same-tank f dom animl)
  (landmap
    (λ (other)
      (=> ((domain-same-tank? dom) animl other)
          (f other)))
    (domain-animals dom)))

; like forall in tank, but only for non-self
(define (forall-others-in-same-tank f dom animl)
  (landmap
    (λ (other)
      (=> (&& ((domain-same-tank? dom) animl other) (not (eq? animl other)))
          (f other)))
    (domain-animals dom)))

; #t iff (f tank animal) is #t for the tank containing animl
(define (for-tank-of f dom animl)
  (landmap
    (λ (t)
      (=> ((domain-in-tank? dom) t animl)
          (f t)))
    (domain-tanks dom)))

; predator?, animal? -> boolean?
(define (can-eat? pred prey)
  (define prey-size (animal-final-size prey))
  (&& (or (equal? (predator-type pred) (animal-type prey))
          (equal? (predator-type pred) (animal-class prey)))
      (or (not (predator-size pred))
          (if (animal-armored? prey)
            (>= (predator-size pred) (+ prey-size prey-size))
            (>= (predator-size pred) prey-size)))))

(define (same-food-type? self other)
  (define self-diet (animal-diet self))
  (define other-diet (animal-diet other))
  (and (&& (food? self-diet) (food? other-diet))
       (equal? (food-type self-diet) (food-type other-diet))))

(define (restriction-satisfied? dom self restr)
  (match restr
   [(shoaler number)
    (>= (count-in-same-tank (curry same-species? self) dom self)
        number)]
   [(predator type size)
    ; assumes no fish can eat itself, which is a safe assumption
    (forall-in-same-tank
      (λ (other) (not (can-eat? restr other)))
      dom
      self)]
   [(active-swimmer)
    (for-tank-of
      (λ (tnk)
        (>= (tank-size tnk)
            ; in the interest of better integer arithmetic, have to use addition
            (let ([s (animal-final-size self)])
              ;multiplier is always 6
              (+ s s s s s s))))
      dom
      self)]
   [(dislikes-conspecifics)
    (forall-others-in-same-tank
      (λ (other) (not (same-species? self other)))
      dom
      self)]
   [(dislikes-congeners)
    (forall-others-in-same-tank
      (λ (other) (not (same-type? self other)))
      dom
      self)]
   [(only-congeners)
    (forall-others-in-same-tank
      (λ (other) (same-type? self other))
      dom
      self)]
   [(rounded-tank)
    (for-tank-of
      (λ (tnk) (tnktyp-rounded? (tank-type tnk)))
      dom
      self)]
   [(dislikes-food-competitors)
    (forall-in-same-tank
      (λ (other) (=> (same-food-type? self other) (same-species? self other)))
      dom
      self)]
   [(wimp)
    (forall-in-same-tank
      ; assumes that the species are concrete for this to be even slightly efficient
      (λ (other) (not (ormap bully? (animal-properties other))))
      dom
      self)]
   [(dislikes-light)
    (for-tank-of
      (λ (tnk) (= 0 (tank-lighting tnk)))
      dom
      self)]
   [(requires-light amt)
    (for-tank-of
      (λ (tnk) (>= (tank-lighting tnk) amt))
      dom
      self)]
   [_
    (error (format "unimplemented restriction ~a" restr))]))

; checks global tank constraints like temperature and size
(define (tank-constraints-satisfied? dom tnk)
  (&&
    ; check temperature of all animals matches the tank's
    (let ([env (tank-environment tnk)])
      (forall-in-tank
        (λ (a)
          (define req (animal-environment a))
          (&& (equal? (environment-temperature env) (environment-temperature req))
              (>= (environment-quality env) (environment-quality req))))
        dom
        tnk))
    ; check that the tank size is large enough
    (>= (tank-size tnk)
        (sum-in-tank (λ (a) (animal-final-size a)) dom tnk))))

(module+ test
  (define-simple-check (check-tank-ok? expected tnk animals)
    (define dom (make-concrete-domain (list (cons tnk animals))))
    (equal? expected (tank-constraints-satisfied? dom tnk)))

  (define basic-tnktyp (make-test-tnktyp))
  (define rounded-tnktyp (make-test-tnktyp #:rounded? #t))

  (define (make-simple-tank
            #:size sz
            #:type [typ basic-tnktyp]
            #:lighting [lght 0]
            #:temp [temp warm-water]
            #:quality [qlty 100])
    (define env (environment temp qlty))
    (make-tank #:id 1 #:name "T" #:type typ #:size sz #:environment env #:lighting lght))

  (test-case "Empty tank is okay"
    (for ([sz '(1 10 50)])
      (check-tank-ok? #t (make-simple-tank #:size sz) '())))

  (test-case "Single fish fits if <= tank size"
    (define (f sz) (list (make-animal (make-simple-species 'typ (make-size sz)))))
    (define (t sz) (make-simple-tank #:size sz))

    (check-tank-ok? #t (t 6) (f 1))
    (check-tank-ok? #t (t 6) (f 5))
    (check-tank-ok? #t (t 6) (f 6))
    (check-tank-ok? #f (t 6) (f 7))
    (check-tank-ok? #f (t 6) (f 234545))
    (check-tank-ok? #t (t 32) (f 31))
    (check-tank-ok? #t (t 32) (f 32))
    (check-tank-ok? #f (t 32) (f 33)))

  (test-case "Multiple fish fit if total <= tank size"
    (define (f . szs) (map (λ (sz) (make-animal (make-simple-species 'typ (make-size sz)))) szs))
    (define (t sz) (make-simple-tank #:size sz))
    (check-tank-ok? #t (t 6) (f 1 1 1 1 1))
    (check-tank-ok? #t (t 6) (f 1 1 1 1 1 1))
    (check-tank-ok? #f (t 6) (f 1 1 1 1 1 1 1))
    (check-tank-ok? #t (t 20) (f 14 6))
    (check-tank-ok? #f (t 19) (f 14 6)))

  (test-case "Armor does not count for tank size requirements"
    (define (f armored? sz)
      (list (make-animal (make-simple-species 'typ (make-size sz #:armored? armored?)))))
    (define (t sz) (make-simple-tank #:size sz))
    (for ([armored? '(#t #f)])
      (check-tank-ok? #t (t 6) (f armored? 5))
      (check-tank-ok? #t (t 6) (f armored? 6))
      (check-tank-ok? #f (t 6) (f armored? 7))))

  (test-case "Only final fish stage is considered for tank size requirements"
    (define (f . szs)
      (define stages (map (λ (sz) (species-stage sz 1)) szs))
      (list (make-animal (make-simple-species 'typ (size stages #f)))))
    (define (t sz) (make-simple-tank #:size sz))
    (check-tank-ok? #t (t 8) (f 3 7))
    (check-tank-ok? #t (t 7) (f 3 7))
    (check-tank-ok? #f (t 6) (f 3 7))
    (check-tank-ok? #f (t 10) (f 1 2 3 4 5 6 7 8 9 10 11))
    (check-tank-ok? #t (t 10) (f 1 2 3 4 5 6 7 8 9 10)))

  (test-case "Tank okay if all temperature matches for all fish"
    (define (f . temps)
      (map (λ (t) (make-animal (make-simple-species 'typ (environment t 50)))) temps))
    (define (t temp) (make-simple-tank #:size 1000000 #:temp temp))
    (check-tank-ok? #t (t warm-water) (f warm-water))
    (check-tank-ok? #t (t cold-water) (f cold-water))
    (check-tank-ok? #f (t warm-water) (f cold-water))
    (check-tank-ok? #f (t cold-water) (f warm-water))
    (check-tank-ok? #t (t warm-water) (f warm-water warm-water warm-water))
    (check-tank-ok? #f (t warm-water) (f cold-water warm-water warm-water))
    (check-tank-ok? #f (t warm-water) (f warm-water cold-water warm-water))
    (check-tank-ok? #f (t warm-water) (f warm-water warm-water cold-water))
    (check-tank-ok? #t (t cold-water) (f cold-water cold-water cold-water))
    (check-tank-ok? #f (t cold-water) (f warm-water cold-water cold-water))
    (check-tank-ok? #f (t cold-water) (f cold-water warm-water cold-water))
    (check-tank-ok? #f (t cold-water) (f cold-water cold-water warm-water)))

  (test-case "Tank okay if quality >= all fish required qualities"
    (define (f . qltys)
      (map (λ (q) (make-animal (make-simple-species 'typ (environment warm-water q)))) qltys))
    (define (t qlty) (make-simple-tank #:size 1000000 #:temp warm-water #:quality qlty))
    (check-tank-ok? #t (t 50) (f 49))
    (check-tank-ok? #t (t 50) (f 50))
    (check-tank-ok? #f (t 50) (f 51))
    (check-tank-ok? #t (t 50) (f 49 30 17 28))
    (check-tank-ok? #f (t 45) (f 49 30 17 28))
    (check-tank-ok? #t (t 100) (f 19 60 87 50))
    (check-tank-ok? #f (t 85) (f 19 60 87 50))
    (check-tank-ok? #f (t 55) (f 19 60 87 50))))

(define (all-constraints-satisfied? dom)
  (&&
    #t
    (landmap
      (curry tank-constraints-satisfied? dom)
      (domain-tanks dom))
    (landmap
      (λ (a)
        (landmap
          (curry restriction-satisfied? dom a)
          (animal-restrictions a)))
      (domain-animals dom))))

; domain? -> (listof (pairof (or/c animal? #f) string?))
(define (find-violated-tank-constraints dom)
  (append-map
    (λ (tnk)
      (define env (tank-environment tnk))
      (append
        (append-map
          (λ (a)
            (define req (animal-environment a))
            (maybe-list ((domain-in-tank? dom) tnk a)
              (append
                (maybe-not-singleton (equal? (environment-temperature env) (environment-temperature req))
                  (cons a "different temperature"))
                (maybe-not-singleton (>= (environment-quality env) (environment-quality req))
                  (cons a "higher water quality")))))
          (domain-animals dom))
        ; check that the tank size is large enough
        (maybe-not-singleton (>= (tank-size tnk) (sum-in-tank (λ (a) (animal-final-size a)) dom tnk))
          (cons #f "tank is not large enough"))))
    (domain-tanks dom)))

; domain? -> (listof (pairof animal? restriction?))
(define (find-violated-restrictions dom)
  (append-map
    (λ (a)
      (map (curry cons a)
           (filter-not (curry restriction-satisfied? dom a)
                       (animal-restrictions a))))
    (domain-animals dom)))

