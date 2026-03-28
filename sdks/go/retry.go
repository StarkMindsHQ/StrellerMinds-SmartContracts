package strellerminds

import (
	"errors"
	"math"
	"math/rand"
	"strings"
	"time"
)

type RetryOptions struct {
	Retries       int
	InitialDelay  time.Duration
	MaxDelay      time.Duration
	Multiplier    float64
	Jitter        bool
	OnRetry       func(err error, attempt int, delay time.Duration)
	IsRetryable   func(err error) bool
}

func defaultRetryOptions() RetryOptions {
	return RetryOptions{
		Retries:      5,
		InitialDelay: 200 * time.Millisecond,
		MaxDelay:     10 * time.Second,
		Multiplier:   2.0,
		Jitter:       true,
	}
}

func Retry[T any](task func() (T, error), opts *RetryOptions) (T, error) {
	var zero T
	options := defaultRetryOptions()
	if opts != nil {
		options = *opts
	}
	attempt := 0
	delay := options.InitialDelay
	var lastErr error

	for attempt <= options.Retries {
		res, err := task()
		if err == nil {
			return res, nil
		}
		lastErr = err
		retryable := true
		if options.IsRetryable != nil {
			retryable = options.IsRetryable(err)
		}
		if !retryable || attempt == options.Retries {
			return zero, err
		}
		computed := delay
		if options.Jitter {
			j := rand.Float64() + 1.0 // [1.0, 2.0)
			computed = time.Duration(math.Min(float64(options.MaxDelay), float64(delay)*j))
		}
		if options.OnRetry != nil {
			func() {
				defer func() { _ = recover() }()
				options.OnRetry(err, attempt+1, computed)
			}()
		}
		time.Sleep(computed)
		delay = time.Duration(math.Min(float64(options.MaxDelay), math.Max(1, float64(delay)*options.Multiplier)))
		attempt++
	}
	if lastErr != nil {
		return zero, lastErr
	}
	return zero, errors.New("retry failed with unknown error")
}

func IsTransientNetworkError(err error) bool {
	if err == nil {
		return false
	}
	msg := strings.ToLower(err.Error())
	substrings := []string{
		"timeout",
		"network",
		"temporary",
		"connection reset",
		"econnreset",
		"econnrefused",
		"503",
		"502",
		"429",
		"rate limit",
		"not_found",
	}
	for _, s := range substrings {
		if strings.Contains(msg, s) {
			return true
		}
	}
	return false
}

